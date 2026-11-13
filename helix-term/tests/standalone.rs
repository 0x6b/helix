use std::{fs, process::Command};

#[test]
fn japanese_word_motion_is_preserved() {
    use helix_core::{movement, Range, Rope};

    for (text, short_end, long_end) in [("漢字かなカナ 次", 2, 7), ("𰀀𰀁かなカナ 次", 2, 7)]
    {
        let text = Rope::from(text);
        assert_eq!(
            movement::move_next_word_start(text.slice(..), Range::point(0), 1),
            Range::new(0, short_end),
        );
        assert_eq!(
            movement::move_next_long_word_start(text.slice(..), Range::point(0), 1),
            Range::new(0, long_end),
        );
    }
}

#[test]
fn copied_binary_uses_bundled_runtime_and_preserves_overrides() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let temporary = root.path().join("tmp");
    fs::create_dir(&home).unwrap();
    fs::create_dir(&temporary).unwrap();
    let binary = root
        .path()
        .join(format!("hx{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_hx"), &binary).unwrap();

    let health = |language: &str| {
        let output = Command::new(&binary)
            .current_dir(root.path())
            .env_clear()
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("APPDATA", home.join("config"))
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("TMPDIR", &temporary)
            .env("TMP", &temporary)
            .env("TEMP", &temporary)
            .args(["--health", language])
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(fs::read_dir(&temporary).unwrap().count(), 0);
        String::from_utf8(output.stdout).unwrap()
    };

    // Race first use in separate processes sharing an initially empty cache.
    std::thread::scope(|scope| {
        for _ in 0..3 {
            scope.spawn(|| {
                let output = health("markdown");
                assert!(
                    output.lines().any(|line| {
                        line.starts_with("Tree-sitter parser:") && line.contains('✓')
                    }),
                    "{output}"
                );
            });
        }
    });
    let caches: Vec<_> = fs::read_dir(home.join("cache/helix/runtime"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(caches.len(), 1);
    let grammar = caches[0].join(format!(
        "grammars/markdown.{}",
        std::env::consts::DLL_EXTENSION
    ));
    let modified = grammar.metadata().unwrap().modified().unwrap();

    for language in ["markdown", "markdown.inline", "rust"] {
        let output = health(language);
        for prefix in ["Tree-sitter parser:", "Highlight queries:"] {
            assert!(
                output
                    .lines()
                    .any(|line| line.starts_with(prefix) && line.contains('✓')),
                "{output}"
            );
        }
    }
    assert_eq!(grammar.metadata().unwrap().modified().unwrap(), modified);
    // Deleting a cache is safe between runs: the standalone binary rebuilds it.
    fs::remove_dir_all(&caches[0]).unwrap();
    let output = health("markdown");
    assert!(
        output
            .lines()
            .any(|line| { line.starts_with("Tree-sitter parser:") && line.contains('✓') }),
        "{output}"
    );

    let config = home.join("config/helix");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("languages.toml"),
        "[[language]]\nname = 'markdown'\nlanguage-servers = []\n",
    )
    .unwrap();
    let output = health("markdown");
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("Configured language servers:") && line.contains("None")),
        "{output}"
    );

    // An explicit external grammar still wins, even if it cannot be loaded.
    // Silently substituting the bundled version would hide a broken override.
    let grammars = root.path().join("runtime/grammars");
    fs::create_dir_all(&grammars).unwrap();
    fs::write(
        grammars.join(format!("markdown.{}", std::env::consts::DLL_EXTENSION)),
        b"not a native grammar",
    )
    .unwrap();
    let output = health("markdown");
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("Tree-sitter parser:") && line.contains("None")),
        "{output}"
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("Highlight queries:") && line.contains('✓')),
        "{output}"
    );
}
