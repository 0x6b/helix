//! Build-time inputs for the standalone distribution, isolated from version metadata.

pub fn prepare() {
    println!("cargo:rerun-if-env-changed=HELIX_CONFIG_DIR");
    let config_dir = std::env::var_os("HELIX_CONFIG_DIR").map(std::path::PathBuf::from);
    if let Some(dir) = &config_dir {
        assert!(
            dir.is_absolute(),
            "HELIX_CONFIG_DIR must be an absolute path"
        );
        assert!(dir.is_dir(), "HELIX_CONFIG_DIR must point to a directory");
        println!("cargo:rerun-if-changed={}", dir.display());
    }
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    for name in ["config.toml", "languages.toml"] {
        let contents = config_dir.as_ref().map_or_else(String::new, |dir| {
            let path = dir.join(name);
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    toml::from_str::<toml::Value>(&contents)
                        .unwrap_or_else(|error| panic!("Invalid {}: {error}", path.display()));
                    contents
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(error) => panic!("Cannot read {}: {error}", path.display()),
            }
        });
        std::fs::write(output.join(name), contents).unwrap();
    }
}
