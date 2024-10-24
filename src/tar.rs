use anyhow::Result;
use log::warn;
use lz4::EncoderBuilder;
use std::{fs::File, io::Write, path::Path};
use tar::Builder;
use walkdir::WalkDir;

pub fn create_tar_lz4(
    base_path: &str,
    file_path: &str,
    include_paths: &[&str],
    exclude_files: &[&str],
) -> Result<()> {
    if Path::new(&file_path).exists() {
        warn!("File {} already exists. Skip archiving.", file_path);
        return Ok(());
    }

    let output_file = File::create(file_path)?;
    let mut encoder = EncoderBuilder::new().build(output_file)?;
    {
        let mut tar = Builder::new(&mut encoder);

        for include_path in include_paths {
            let full_path = Path::new(base_path).join(include_path);
            add_to_tar(&mut tar, &full_path, include_path, exclude_files)?;
        }

        tar.finish()?;
    }
    let (_output, result) = encoder.finish();
    result?;

    Ok(())
}

fn add_to_tar<W: Write>(
    tar: &mut Builder<W>,
    full_path: &Path,
    base_path: &str,
    exclude_files: &[&str],
) -> Result<()> {
    let walker = WalkDir::new(full_path).into_iter();

    for entry in walker.filter_entry(|e| {
        !exclude_files
            .iter()
            .any(|ex| e.path().to_str().unwrap().ends_with(ex))
    }) {
        let entry = entry?;
        let path = entry.path();
        if path == full_path {
            continue;
        }

        let relative = path.strip_prefix(full_path)?;
        let tar_path = Path::new(base_path).join(relative);

        if path.is_file() {
            let mut file = File::open(path)?;
            tar.append_file(tar_path, &mut file)?;
        } else if path.is_dir() {
            tar.append_dir(tar_path, path)?;
        }
    }
    Ok(())
}
