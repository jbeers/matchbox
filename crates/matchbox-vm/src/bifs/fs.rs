#[cfg(feature = "bif-io")]
use crate::types::{BxVM, BxValue};
#[cfg(feature = "bif-io")]
use std::fs;
#[cfg(all(feature = "bif-io", unix))]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(feature = "bif-io")]
use std::path::Path;
#[cfg(feature = "bif-io")]
use uuid::Uuid;

#[cfg(feature = "bif-io")]
use walkdir::WalkDir;

#[cfg(feature = "bif-io")]
fn normalize_path_string(path: &Path) -> String {
    normalize_path_text(path.to_string_lossy().as_ref())
}

#[cfg(feature = "bif-io")]
fn normalize_path_text(path: &str) -> String {
    #[cfg(windows)]
    {
        if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", stripped);
        }
        if let Some(stripped) = path.strip_prefix(r"\\?\") {
            return stripped.to_string();
        }
    }

    path.to_string()
}

#[cfg(feature = "bif-io")]
pub fn directory_exists(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("directoryExists() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);
    Ok(BxValue::new_bool(path.exists() && path.is_dir()))
}

#[cfg(feature = "bif-io")]
pub fn directory_create(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("directoryCreate() expects at least 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);

    let recurse = if args.len() > 1 {
        args[1].as_bool()
    } else {
        true
    };

    if recurse {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    } else {
        fs::create_dir(path).map_err(|e| e.to_string())?;
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn directory_delete(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("directoryDelete() expects at least 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);

    let recurse = if args.len() > 1 {
        args[1].as_bool()
    } else {
        false
    };

    if recurse {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    } else {
        fs::remove_dir(path).map_err(|e| e.to_string())?;
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn directory_list(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("directoryList() expects at least 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let recurse = if args.len() > 1 {
        args[1].as_bool()
    } else {
        false
    };

    let array_id = vm.array_new();

    let walker = if recurse {
        WalkDir::new(&path_str).min_depth(1)
    } else {
        WalkDir::new(&path_str).min_depth(1).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let p = entry.path().to_string_lossy().to_string();
        let s_id = vm.string_new(p);
        vm.array_push(array_id, BxValue::new_ptr(s_id));
    }

    Ok(BxValue::new_ptr(array_id))
}

#[cfg(feature = "bif-io")]
pub fn file_exists(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileExists() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);
    // Use symlink_metadata to detect existence including symlinks
    Ok(BxValue::new_bool(path.symlink_metadata().is_ok()))
}

#[cfg(feature = "bif-io")]
pub fn file_delete(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileDelete() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    fs::remove_file(path_str).map_err(|e| e.to_string())?;
    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_move(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileMove() expects 2 arguments: (source, destination)".to_string());
    }
    let src = vm.to_string(args[0]);
    let dest = vm.to_string(args[1]);

    fs_extra::file::move_file(&src, &dest, &fs_extra::file::CopyOptions::new())
        .map_err(|e| e.to_string())?;

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_publish_exclusive(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "filePublishExclusive() expects 2 arguments: (source, destination)".to_string(),
        );
    }
    let src = vm.to_string(args[0]);
    let dest = vm.to_string(args[1]);

    // hard_link publishes the complete same-filesystem file atomically and fails if dest exists.
    fs::hard_link(&src, &dest).map_err(|e| e.to_string())?;
    if let Err(error) = fs::remove_file(&src) {
        let _ = fs::remove_file(&dest);
        return Err(error.to_string());
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_copy(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileCopy() expects 2 arguments: (source, destination)".to_string());
    }
    let src = vm.to_string(args[0]);
    let dest = vm.to_string(args[1]);

    fs::copy(src, dest).map_err(|e| e.to_string())?;

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_info(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileInfo() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;

    let struct_id = vm.struct_new();

    vm.struct_set(
        struct_id,
        "size",
        BxValue::new_number(metadata.len() as f64),
    );
    let type_id = vm.string_new(if metadata.is_dir() { "Dir" } else { "File" }.to_string());
    vm.struct_set(struct_id, "type", BxValue::new_ptr(type_id));
    vm.struct_set(
        struct_id,
        "is_directory",
        BxValue::new_bool(metadata.is_dir()),
    );
    vm.struct_set(struct_id, "is_file", BxValue::new_bool(metadata.is_file()));
    vm.struct_set(
        struct_id,
        "is_symlink",
        BxValue::new_bool(metadata.file_type().is_symlink()),
    );
    vm.struct_set(
        struct_id,
        "is_readonly",
        BxValue::new_bool(metadata.permissions().readonly()),
    );

    if metadata.file_type().is_symlink() {
        if let Ok(target) = fs::read_link(path) {
            let target_str = target.to_string_lossy().to_string();
            let s_id = vm.string_new(target_str);
            vm.struct_set(struct_id, "target", BxValue::new_ptr(s_id));
        }
    }

    Ok(BxValue::new_ptr(struct_id))
}

#[cfg(feature = "bif-io")]
pub fn file_create_symlink(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileCreateSymlink() expects 2 arguments: (link, target)".to_string());
    }
    let link = vm.to_string(args[0]);
    let target = vm.to_string(args[1]);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    {
        let target_path = Path::new(&target);
        if target_path.is_dir() {
            std::os::windows::fs::symlink_dir(target, link).map_err(|e| e.to_string())?;
        } else {
            std::os::windows::fs::symlink_file(target, link).map_err(|e| e.to_string())?;
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        return Err("Symlinks not supported on this platform".to_string());
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_set_executable(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileSetExecutable() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path_str)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path_str, perms).map_err(|e| e.to_string())?;
    }

    #[cfg(not(unix))]
    {
        // No-op on Windows for executable bit
        let _ = path_str;
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_read(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileRead() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let content = fs::read_to_string(path_str).map_err(|e| e.to_string())?;
    let s_id = vm.string_new(content);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn file_write(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileWrite() expects 2 arguments: (path, content)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let content = vm.to_string(args[1]);
    fs::write(path_str, content).map_err(|e| e.to_string())?;
    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_write_exclusive(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "fileWriteExclusive() expects at least 2 arguments: (path, content, mode)".to_string(),
        );
    }
    let path = vm.to_string(args[0]);
    let content = vm.to_string(args[1]);
    let mode = args
        .get(2)
        .map(|value| vm.to_string(*value))
        .unwrap_or_else(|| "0600".to_string());
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mode = u32::from_str_radix(&mode, 8)
            .map_err(|_| format!("Invalid POSIX mode string: {mode}"))?;
        options.mode(mode);
    }
    #[cfg(not(unix))]
    let _ = mode;
    let mut file = options
        .open(&path)
        .map_err(|error| format!("Could not exclusively create file '{path}': {error}"))?;
    use std::io::Write;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("Could not write file '{path}': {error}"))?;
    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_append(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileAppend() expects 2 arguments: (path, content)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let content = vm.to_string(args[1]);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_str)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn contract_path(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("contractPath() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path_text = normalize_path_text(&path_str);
    let path = Path::new(&path_text);
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd_text = normalize_path_string(&cwd);
    let cwd = Path::new(&cwd_text);
    let relative = path.strip_prefix(cwd).unwrap_or(path);
    let result = relative.to_string_lossy().to_string();
    let result = if result.is_empty() {
        ".".to_string()
    } else {
        result
    };
    let s_id = vm.string_new(result);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn get_temp_directory(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if !args.is_empty() {
        return Err("getTempDirectory() expects no arguments".to_string());
    }

    let mut directory = std::env::temp_dir().to_string_lossy().to_string();
    if !directory.ends_with(std::path::MAIN_SEPARATOR) {
        directory.push(std::path::MAIN_SEPARATOR);
    }
    Ok(BxValue::new_ptr(vm.string_new(directory)))
}

#[cfg(feature = "bif-io")]
pub fn create_temp_directory(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let directory = if !args.is_empty() {
        vm.to_string(args[0])
    } else {
        std::env::temp_dir().to_string_lossy().to_string()
    };
    let prefix = if args.len() > 1 {
        vm.to_string(args[1])
    } else {
        String::new()
    };

    let base = Path::new(&directory);
    if !base.exists() {
        fs::create_dir_all(base).map_err(|e| e.to_string())?;
    }

    let mut name = if prefix.is_empty() {
        "tmp".to_string()
    } else {
        prefix
    };
    let random_suffix: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    name.push_str(&format!("_{}", random_suffix));

    let dir_path = base.join(&name);
    fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;
    let canonical = normalize_path_string(&dir_path.canonicalize().unwrap_or(dir_path));
    let s_id = vm.string_new(canonical);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn create_temp_file(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let directory = if !args.is_empty() {
        vm.to_string(args[0])
    } else {
        std::env::temp_dir().to_string_lossy().to_string()
    };
    let prefix = if args.len() > 1 {
        vm.to_string(args[1])
    } else {
        String::new()
    };
    let suffix = if args.len() > 2 {
        vm.to_string(args[2])
    } else {
        ".tmp".to_string()
    };
    if prefix.contains('/')
        || prefix.contains('\\')
        || suffix.contains('/')
        || suffix.contains('\\')
    {
        return Err(
            "createTempFile() prefix and suffix must not contain path separators".to_string(),
        );
    }

    let base = Path::new(&directory);
    if !base.exists() {
        fs::create_dir_all(base).map_err(|e| e.to_string())?;
    }

    let stem = if prefix.is_empty() {
        "tmp".to_string()
    } else {
        prefix
    };
    let mut file_path = None;
    for _ in 0..16 {
        let candidate = base.join(format!("{}_{}{}", stem, Uuid::new_v4().simple(), suffix));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&candidate) {
            Ok(_) => {
                file_path = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    let file_path =
        file_path.ok_or_else(|| "Could not create a unique temporary file".to_string())?;
    let canonical = file_path
        .canonicalize()
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();
    let s_id = vm.string_new(canonical);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn directory_copy(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "directoryCopy() expects at least 2 arguments: (source, destination)".to_string(),
        );
    }
    let source = vm.to_string(args[0]);
    let dest = vm.to_string(args[1]);
    let recurse = if args.len() > 2 {
        args[2].as_bool()
    } else {
        false
    };
    let create_path = if args.len() > 4 {
        args[4].as_bool()
    } else {
        true
    };
    let overwrite = if args.len() > 5 {
        args[5].as_bool()
    } else {
        false
    };

    let src_path = Path::new(&source);
    if !src_path.exists() || !src_path.is_dir() {
        return Err(format!("Source directory does not exist: {}", source));
    }

    let dest_path = Path::new(&dest);
    if create_path && !dest_path.exists() {
        fs::create_dir_all(dest_path).map_err(|e| e.to_string())?;
    }

    if recurse {
        for entry in WalkDir::new(src_path).min_depth(1) {
            let entry = entry.map_err(|e| e.to_string())?;
            let rel = entry
                .path()
                .strip_prefix(src_path)
                .map_err(|e| e.to_string())?;
            let target = dest_path.join(rel);
            if entry.file_type().is_dir() {
                if !target.exists() {
                    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                }
            } else {
                if target.exists() && !overwrite {
                    continue;
                }
                if let Some(parent) = target.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                }
                fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
            }
        }
    } else {
        for entry in fs::read_dir(src_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let target = dest_path.join(entry.file_name());
            if entry.file_type().map_err(|e| e.to_string())?.is_file() {
                if target.exists() && !overwrite {
                    continue;
                }
                fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(BxValue::new_null())
}

#[cfg(feature = "bif-io")]
pub fn directory_move(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("directoryMove() expects 2 arguments: (source, destination)".to_string());
    }
    let source = vm.to_string(args[0]);
    let dest = vm.to_string(args[1]);
    let create_path = if args.len() > 2 {
        args[2].as_bool()
    } else {
        true
    };

    let dest_path = Path::new(&dest);
    if create_path {
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }

    fs::rename(&source, &dest).map_err(|e| e.to_string())?;
    Ok(BxValue::new_null())
}

#[cfg(feature = "bif-io")]
pub fn expand_path(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("expandPath() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);
    let has_trailing = path_str.ends_with('/') || path_str.ends_with('\\');

    let expanded = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };

    let resolved = expanded.canonicalize().unwrap_or(expanded);
    let mut result = normalize_path_string(&resolved);
    if has_trailing && !result.ends_with(std::path::MAIN_SEPARATOR) {
        result.push(std::path::MAIN_SEPARATOR);
    }
    let s_id = vm.string_new(result);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn file_close(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileClose() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn file_get_mime_type(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("fileGetMimeType() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let strict = if args.len() > 1 {
        args[1].as_bool()
    } else {
        true
    };

    let path = Path::new(&path_str);
    if strict {
        if !path.exists() {
            return Err(format!("The file [{}] does not exist", path_str));
        }
        let meta = fs::metadata(path).map_err(|e| e.to_string())?;
        if meta.len() == 0 {
            return Err(format!("The file [{}] is empty", path_str));
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "bxs" | "bx" => "text/plain",
        "rs" => "text/plain",
        "java" => "text/plain",
        "md" => "text/markdown",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        _ => "application/octet-stream",
    };

    let s_id = vm.string_new(mime.to_string());
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn file_is_eof(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileIsEOF() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn file_open(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileOpen() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn file_read_line(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileReadLine() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn file_seek(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileSeek() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn file_set_access_mode(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileSetAccessMode() expects 2 arguments: (path, mode)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let mode_str = vm.to_string(args[1]);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = u32::from_str_radix(&mode_str, 8)
            .map_err(|_| format!("Invalid POSIX mode string: {}", mode_str))?;
        let mut perms = fs::metadata(&path_str)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(mode);
        fs::set_permissions(&path_str, perms).map_err(|e| e.to_string())?;
    }

    #[cfg(not(unix))]
    {
        let _ = (path_str, mode_str);
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_set_attribute(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileSetAttribute() expects 2 arguments: (path, attribute)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let attribute = vm.to_string(args[1]).to_lowercase();

    let path = Path::new(&path_str);
    if !path.exists() {
        return Err(format!("File does not exist: {}", path_str));
    }

    match attribute.as_str() {
        "normal" | "default" => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(0o664);
                fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
            }
        }
        "readonly" => {
            let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
            perms.set_readonly(true);
            fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
        }
        "archive" | "hidden" | "system" => {}
        _ => {
            return Err(format!(
                "The attribute provided [{}] is not valid",
                attribute
            ));
        }
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_set_last_modified(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("fileSetLastModified() expects 2 arguments: (path, date)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let date_str = vm.to_string(args[1]);

    let epoch_millis: f64 = if let Ok(n) = date_str.parse::<f64>() {
        n
    } else {
        return Err(format!("Cannot parse date: {}", date_str));
    };

    let duration = std::time::Duration::from_millis(epoch_millis as u64);
    let system_time = std::time::UNIX_EPOCH + duration;
    let ft = filetime::FileTime::from_system_time(system_time);
    filetime::set_file_mtime(&path_str, ft).map_err(|e| e.to_string())?;

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-io")]
pub fn file_write_line(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("fileWriteLine() is not yet implemented: file handle infrastructure required".to_string())
}

#[cfg(feature = "bif-io")]
pub fn get_canonical_path(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("getCanonicalPath() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);

    match path.canonicalize() {
        Ok(canonical) => {
            let mut result = normalize_path_string(&canonical);
            if canonical.is_dir() && !result.ends_with(std::path::MAIN_SEPARATOR) {
                result.push(std::path::MAIN_SEPARATOR);
            }
            let s_id = vm.string_new(result);
            Ok(BxValue::new_ptr(s_id))
        }
        Err(_) => {
            let s_id = vm.string_new(path_str);
            Ok(BxValue::new_ptr(s_id))
        }
    }
}

#[cfg(feature = "bif-io")]
pub fn get_directory_from_path(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("getDirectoryFromPath() expects 1 argument".to_string());
    }
    let path_str = vm.to_string(args[0]);

    if path_str.is_empty() {
        let s_id = vm.string_new("/".to_string());
        return Ok(BxValue::new_ptr(s_id));
    }

    if path_str.ends_with('/') || path_str.ends_with('\\') {
        let s_id = vm.string_new(path_str);
        return Ok(BxValue::new_ptr(s_id));
    }

    let last_sep = path_str.rfind('/').or_else(|| path_str.rfind('\\'));
    let result = match last_sep {
        Some(idx) => path_str[..=idx].to_string(),
        None => "/".to_string(),
    };

    let s_id = vm.string_new(result);
    Ok(BxValue::new_ptr(s_id))
}

#[cfg(feature = "bif-io")]
pub fn property_file(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        let struct_id = vm.struct_new();
        return Ok(BxValue::new_ptr(struct_id));
    }
    let path_str = vm.to_string(args[0]);
    if path_str.is_empty() {
        let struct_id = vm.struct_new();
        return Ok(BxValue::new_ptr(struct_id));
    }

    let content = fs::read_to_string(&path_str)
        .map_err(|e| format!("Cannot read property file [{}]: {}", path_str, e))?;

    let struct_id = vm.struct_new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let k = key.trim();
            let v = value.trim();
            let v_id = vm.string_new(v.to_string());
            vm.struct_set(struct_id, k, BxValue::new_ptr(v_id));
        }
    }

    Ok(BxValue::new_ptr(struct_id))
}
