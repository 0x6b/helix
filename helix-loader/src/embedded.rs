//! Standalone distribution data and cache. Upstream loaders only call the hooks here.

use std::path::{Path, PathBuf};

pub const CONFIG: &str = include_str!(concat!(env!("OUT_DIR"), "/config.toml"));
const LANGUAGES: &str = include_str!(concat!(env!("OUT_DIR"), "/languages.toml"));

static RUNTIME: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();

/// Install the bundled runtime as the lowest-priority fallback before any lookup.
pub fn initialize_runtime(archive: &[u8], hash: &str) -> anyhow::Result<()> {
    let dir = cache_runtime(archive, hash, &crate::cache_dir().join("runtime"))?;
    RUNTIME
        .set(dir)
        .map_err(|_| anyhow::anyhow!("Embedded runtime is already initialized"))
}

pub(crate) fn runtime_dirs() -> impl Iterator<Item = PathBuf> {
    RUNTIME
        .get()
        .into_iter()
        .flat_map(|dir| [dir.join("profile"), dir.clone()])
}

pub(crate) fn language_defaults() -> Result<toml::Value, toml::de::Error> {
    Ok(crate::merge_toml_values(
        crate::config::default_lang_config(),
        toml::from_str(LANGUAGES)?,
        3,
    ))
}

fn cache_runtime(archive: &[u8], hash: &str, cache: &Path) -> anyhow::Result<PathBuf> {
    let destination = cache.join(hash);
    if destination.is_dir() {
        return Ok(destination);
    }
    std::fs::create_dir_all(cache)?;
    let mut builder = tempfile::Builder::new();
    builder.prefix(".extract-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    let dir = builder.tempdir_in(cache)?;
    tar::Archive::new(flate2::read::GzDecoder::new(archive)).unpack(dir.path())?;
    // Publish only complete runtimes. Never remove a cache another editor uses.
    if let Err(error) = std::fs::rename(dir.path(), &destination) {
        if !destination.is_dir() {
            return Err(error.into());
        }
    }
    Ok(destination)
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_runtime_cache_is_private_complete_and_reusable() {
        use std::io::Write;

        let mut archive = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(7);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, "bundled-test-file", &b"bundled"[..])
            .unwrap();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&archive.into_inner().unwrap()).unwrap();
        let archive = encoder.finish().unwrap();
        let root = tempfile::tempdir().unwrap();
        let dir = super::cache_runtime(&archive, "test-runtime", root.path()).unwrap();
        super::RUNTIME.set(dir.clone()).unwrap();
        assert_eq!(
            crate::runtime_file("bundled-test-file"),
            dir.join("bundled-test-file")
        );
        assert_eq!(
            std::fs::read(crate::runtime_file("bundled-test-file")).unwrap(),
            b"bundled"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(dir.metadata().unwrap().permissions().mode() & 0o777, 0o700);
        }
        // A marker not present in the bundle survives reuse, proving no rewrite.
        std::fs::write(dir.join("marker"), b"retained").unwrap();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    assert_eq!(
                        super::cache_runtime(&archive, "test-runtime", root.path()).unwrap(),
                        dir
                    );
                });
            }
        });
        assert_eq!(std::fs::read(dir.join("marker")).unwrap(), b"retained");
        assert!(super::cache_runtime(b"invalid archive", "invalid", root.path()).is_err());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
}
