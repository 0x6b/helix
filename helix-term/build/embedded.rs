//! Standalone runtime packaging. Keep the platform build script independent.

use std::{env, fs::File, path::PathBuf};

use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha256};

pub fn bundle() {
    println!("cargo:rerun-if-env-changed=HELIX_DISABLE_AUTO_GRAMMAR_BUILD");
    println!("cargo:rerun-if-changed=../languages.toml");
    let mut grammars = helix_loader::grammar::embedded::prepare(
        &env::var("TARGET").unwrap(),
        env::var_os("HELIX_DISABLE_AUTO_GRAMMAR_BUILD").is_none(),
    )
    .expect("Failed to prepare bundled tree-sitter grammars");
    grammars.sort_unstable();

    let runtime = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../runtime");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("runtime.tar.gz");
    let encoder = GzEncoder::new(File::create(&output).unwrap(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    archive.mode(tar::HeaderMode::Deterministic);
    for directory in ["queries", "themes"] {
        let path = runtime.join(directory);
        println!("cargo:rerun-if-changed={}", path.display());
        archive.append_dir_all(directory, path).unwrap();
    }
    println!("cargo:rerun-if-env-changed=HELIX_CONFIG_DIR");
    if let Some(dir) = env::var_os("HELIX_CONFIG_DIR") {
        let dir = PathBuf::from(dir);
        assert!(
            dir.is_absolute(),
            "HELIX_CONFIG_DIR must be an absolute path"
        );
        assert!(dir.is_dir(), "HELIX_CONFIG_DIR must point to a directory");
        println!("cargo:rerun-if-changed={}", dir.display());
        let themes = dir.join("themes");
        if themes.exists() {
            let mut paths: Vec<_> = std::fs::read_dir(themes)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
                .collect();
            paths.sort();
            for path in paths {
                let contents = std::fs::read_to_string(&path).unwrap();
                toml::from_str::<toml::Value>(&contents)
                    .unwrap_or_else(|error| panic!("Invalid {}: {error}", path.display()));
                archive
                    .append_path_with_name(
                        &path,
                        PathBuf::from("profile/themes").join(path.file_name().unwrap()),
                    )
                    .unwrap();
            }
        }
    }
    println!("cargo:rerun-if-changed=../runtime/tutor");
    archive
        .append_path_with_name(runtime.join("tutor"), "tutor")
        .unwrap();

    let extension = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "dll",
        "macos" => "dylib",
        _ => "so",
    };
    for grammar in grammars {
        let relative = format!("grammars/{grammar}.{extension}");
        let path = runtime.join(&relative);
        println!("cargo:rerun-if-changed={}", path.display());
        archive.append_path_with_name(&path, relative).unwrap_or_else(|error| {
            panic!("Cannot bundle {}: {error}. All built-in grammars must be built, even with HELIX_DISABLE_AUTO_GRAMMAR_BUILD.", path.display())
        });
    }
    archive.into_inner().unwrap().finish().unwrap();
    let hash: String = Sha256::digest(std::fs::read(output).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("cargo:rustc-env=HELIX_RUNTIME_HASH={hash}");
}
