use anyhow::{Context, Result, bail};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::ZipArchive;

pub fn resolve_pack_root(input: &Path) -> Result<(PathBuf, Option<TempDir>)> {
    if input.is_dir() {
        return Ok((input.to_path_buf(), None));
    }

    let temp_dir = tempfile::tempdir()
        .with_context(|| format!("failed to create temp dir for {}", input.display()))?;
    unzip_gtpack_to_dir(input, temp_dir.path())
        .with_context(|| format!("failed to extract {}", input.display()))?;
    let pack_root = find_pack_root(temp_dir.path()).with_context(|| {
        format!(
            "failed to locate pack root in extracted contents of {}",
            input.display()
        )
    })?;
    Ok((pack_root, Some(temp_dir)))
}

pub fn unzip_gtpack_to_dir(gtpack_path: &Path, out_dir: &Path) -> Result<()> {
    let file = fs::File::open(gtpack_path)
        .with_context(|| format!("failed to open {}", gtpack_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("{} is not a valid gtpack", gtpack_path.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("failed to read {}", gtpack_path.display()))?;
        let Some(enclosed) = entry.enclosed_name() else {
            continue;
        };
        let out_path = out_dir.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .with_context(|| format!("failed to create {}", out_path.display()))?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let mut outfile = fs::File::create(&out_path)
            .with_context(|| format!("failed to create {}", out_path.display()))?;
        io::copy(&mut entry, &mut outfile)
            .with_context(|| format!("failed to write {}", out_path.display()))?;
    }

    Ok(())
}

pub fn find_pack_root(extracted_root: &Path) -> Result<PathBuf> {
    let direct_pack_yaml = extracted_root.join("pack.yaml");
    if direct_pack_yaml.is_file() {
        return Ok(extracted_root.to_path_buf());
    }

    let mut pack_yaml_matches = Vec::new();
    let mut manifest_matches = Vec::new();
    for entry in WalkDir::new(extracted_root) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() == "pack.yaml" {
            pack_yaml_matches.push(entry.path().to_path_buf());
        } else if entry.file_name() == "manifest.cbor" {
            manifest_matches.push(entry.path().to_path_buf());
        }
    }

    pack_yaml_matches.sort();
    if pack_yaml_matches.len() == 1 {
        return Ok(pack_yaml_matches[0]
            .parent()
            .ok_or_else(|| anyhow::anyhow!("pack.yaml has no parent"))?
            .to_path_buf());
    }
    if pack_yaml_matches.len() > 1 {
        let list = pack_yaml_matches
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "multiple pack.yaml files found in extracted contents: {}",
            list
        );
    }

    manifest_matches.sort();
    if manifest_matches.len() == 1 {
        return Ok(manifest_matches[0]
            .parent()
            .ok_or_else(|| anyhow::anyhow!("manifest.cbor has no parent"))?
            .to_path_buf());
    }
    if manifest_matches.len() > 1 {
        let list = manifest_matches
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "multiple manifest.cbor files found in extracted contents: {}",
            list
        );
    }

    bail!("missing pack.yaml or manifest.cbor in gtpack (searched extracted contents)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::FileOptions;

    fn write_zip(path: &Path, entries: &[(&str, &str)]) -> Result<()> {
        let file = fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::<()>::default();
        for (name, content) in entries {
            zip.start_file(name, options)?;
            zip.write_all(content.as_bytes())?;
        }
        zip.finish()?;
        Ok(())
    }

    #[test]
    fn resolves_directory_input() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("pack.yaml"), "name: demo")?;
        let (pack_root, temp_dir) = resolve_pack_root(temp.path())?;
        assert!(temp_dir.is_none());
        assert_eq!(pack_root, temp.path());
        Ok(())
    }

    #[test]
    fn resolves_gtpack_root_pack_yaml() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let gtpack = temp.path().join("demo.gtpack");
        write_zip(&gtpack, &[("pack.yaml", "name: demo")])?;
        let (pack_root, temp_dir) = resolve_pack_root(&gtpack)?;
        let temp_dir = temp_dir.expect("tempdir");
        assert_eq!(pack_root, temp_dir.path());
        assert!(pack_root.join("pack.yaml").is_file());
        Ok(())
    }

    #[test]
    fn resolves_gtpack_nested_pack_yaml() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let gtpack = temp.path().join("nested.gtpack");
        write_zip(&gtpack, &[("nested/pack.yaml", "name: demo")])?;
        let (pack_root, temp_dir) = resolve_pack_root(&gtpack)?;
        let temp_dir = temp_dir.expect("tempdir");
        assert!(pack_root.starts_with(temp_dir.path()));
        assert_eq!(
            pack_root.file_name().and_then(|s| s.to_str()),
            Some("nested")
        );
        Ok(())
    }

    #[test]
    fn resolves_gtpack_manifest_cbor() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let gtpack = temp.path().join("manifest.gtpack");
        write_zip(&gtpack, &[("manifest.cbor", "stub")])?;
        let (pack_root, temp_dir) = resolve_pack_root(&gtpack)?;
        let temp_dir = temp_dir.expect("tempdir");
        assert_eq!(pack_root, temp_dir.path());
        assert!(pack_root.join("manifest.cbor").is_file());
        Ok(())
    }

    #[test]
    fn gtpack_missing_pack_yaml_errors() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let gtpack = temp.path().join("missing.gtpack");
        write_zip(&gtpack, &[("readme.txt", "no pack")])?;
        let err = resolve_pack_root(&gtpack).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("missing pack.yaml or manifest.cbor in gtpack"));
        assert!(msg.contains(&gtpack.display().to_string()));
        Ok(())
    }

    #[test]
    fn gtpack_multiple_pack_yaml_errors() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let gtpack = temp.path().join("multi.gtpack");
        write_zip(
            &gtpack,
            &[("a/pack.yaml", "name: a"), ("b/pack.yaml", "name: b")],
        )?;
        let err = resolve_pack_root(&gtpack).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("multiple pack.yaml files found in extracted contents"));
        assert!(msg.contains("a/pack.yaml"));
        assert!(msg.contains("b/pack.yaml"));
        Ok(())
    }
}
