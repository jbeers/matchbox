#[cfg(feature = "bif-zip")]
use crate::types::{BxVM, BxValue};
#[cfg(feature = "bif-zip")]
use std::fs;
#[cfg(feature = "bif-zip")]
use std::path::Path;
#[cfg(feature = "bif-zip")]
use std::io::Write;

#[cfg(feature = "bif-zip")]
pub fn zip_extract(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("extract() expects 2 arguments: (zip_file, dest_dir)".to_string());
    }
    let zip_file_str = vm.to_string(args[0]);
    let dest_dir_str = vm.to_string(args[1]);

    let file =
        fs::File::open(&zip_file_str).map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read file from zip: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(&dest_dir_str).join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
            }
            let mut outfile =
                fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .map_err(|e| format!("Failed to set permissions: {}", e))?;
            }
        }
    }

    Ok(BxValue::new_bool(true))
}

#[cfg(feature = "bif-zip")]
pub fn zip_compress(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("compress() expects at least 2 arguments: (source, destination)".to_string());
    }
    let source_str = vm.to_string(args[0]);
    let dest_str = vm.to_string(args[1]);
    let overwrite = if args.len() > 2 { args[2].as_bool() } else { false };

    let source_path = Path::new(&source_str);
    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source_str));
    }

    let dest_path = Path::new(&dest_str);
    if dest_path.exists() && !overwrite {
        return Err(format!("Destination already exists: {}", dest_str));
    }

    let file = fs::File::create(&dest_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    if source_path.is_file() {
        let name = source_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        zip.start_file(name, options)
            .map_err(|e| format!("Failed to start file in zip: {}", e))?;
        let data = fs::read(source_path)
            .map_err(|e| format!("Failed to read source file: {}", e))?;
        zip.write_all(&data)
            .map_err(|e| format!("Failed to write to zip: {}", e))?;
    } else if source_path.is_dir() {
        for entry in walkdir::WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let rel = path.strip_prefix(source_path.parent().unwrap_or(source_path))
                    .map_err(|e| format!("Path error: {}", e))?;
                let name = rel.to_string_lossy().to_string();
                zip.start_file(name, options)
                    .map_err(|e| format!("Failed to start file in zip: {}", e))?;
                let data = fs::read(path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                zip.write_all(&data)
                    .map_err(|e| format!("Failed to write to zip: {}", e))?;
            }
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize zip: {}", e))?;
    Ok(BxValue::new_ptr(vm.string_new(dest_str)))
}

#[cfg(feature = "bif-zip")]
pub fn zip_is_zip_file(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("isZipFile() expects 1 argument: (path)".to_string());
    }
    let path_str = vm.to_string(args[0]);
    let path = Path::new(&path_str);
    if !path.exists() {
        return Ok(BxValue::new_bool(false));
    }
    match fs::File::open(path) {
        Ok(file) => {
            match zip::ZipArchive::new(file) {
                Ok(_) => Ok(BxValue::new_bool(true)),
                Err(_) => Ok(BxValue::new_bool(false)),
            }
        }
        Err(_) => Ok(BxValue::new_bool(false)),
    }
}
