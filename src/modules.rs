use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// A resolved, validated module ready for use in compilation.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// The module name (e.g. `"bx-strings"`). Becomes the `bxModules.{name}` namespace.
    pub name: String,
    /// Absolute physical path to the module root directory.
    pub path: PathBuf,
    /// Contents of each `bifs/*.bxs` (or `.bx`) file in this module, injected as extra
    /// prelude sources during tree-shaking.
    pub bif_sources: Vec<String>,
    /// Whether this module contains a `matchbox/Cargo.toml` for native Rust compilation.
    pub has_native: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// matchbox.toml deserialization
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    modules: HashMap<String, ManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    path: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Discover all modules for the current compilation.
///
/// Sources (in priority order — CLI `--module` flags override manifest entries with the same
/// directory name):
/// 1. `matchbox.toml` in `project_dir`
/// 2. `extra_module_paths` collected from `--module <path>` CLI flags
///
/// Returns an empty `Vec` when neither source provides any modules, so callers can always
/// call this unconditionally.
pub fn discover_modules(
    project_dir: &Path,
    extra_module_paths: &[PathBuf],
) -> Result<Vec<ModuleInfo>> {
    // Collect (name, raw_path) pairs.  Later entries for the same name replace earlier ones.
    let mut entries: Vec<(String, PathBuf)> = Vec::new();

    // 1. Read matchbox.toml if present.
    let manifest_path = project_dir.join("matchbox.toml");
    if manifest_path.exists() {
        let text = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
        let manifest: Manifest = toml::from_str(&text)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;
        for (name, entry) in manifest.modules {
            let raw = Path::new(&entry.path);
            let path = if raw.is_absolute() {
                raw.to_path_buf()
            } else {
                project_dir.join(raw)
            };
            entries.push((name, path));
        }
    }

    // 2. --module CLI overrides: derive name from the directory name, replace manifest entry
    //    with the same name so CLI always wins.
    for raw in extra_module_paths {
        let path = if raw.is_absolute() {
            raw.clone()
        } else {
            project_dir.join(raw)
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        entries.retain(|(n, _)| n != &name);
        entries.push((name, path));
    }

    // 3. Validate and resolve each entry.
    let mut modules = Vec::new();
    for (name, path) in entries {
        let path = path.canonicalize().with_context(|| {
            format!("Module '{}': path does not exist: {}", name, path.display())
        })?;

        let descriptor = path.join("ModuleConfig.bx");
        if !descriptor.exists() {
            bail!(
                "Module '{}' at '{}' is missing ModuleConfig.bx",
                name,
                path.display()
            );
        }

        // Collect bifs/*.bxs sources (sorted for determinism).
        let bifs_dir = path.join("bifs");
        let mut bif_sources = Vec::new();
        if bifs_dir.is_dir() {
            let mut bif_files: Vec<_> = std::fs::read_dir(&bifs_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let entry_path = e.path();
                    let ext = entry_path.extension().and_then(|x| x.to_str());
                    matches!(ext, Some("bxs") | Some("bx"))
                })
                .collect();
            bif_files.sort_by_key(|e| e.path());
            for entry in bif_files {
                let src = std::fs::read_to_string(entry.path()).with_context(|| {
                    format!(
                        "Module '{}': failed to read {}",
                        name,
                        entry.path().display()
                    )
                })?;
                bif_sources.push(src);
            }
        }

        let has_native = path.join("matchbox").join("Cargo.toml").exists();

        modules.push(ModuleInfo {
            name,
            path,
            bif_sources,
            has_native,
        });
    }

    Ok(modules)
}
