//! Configuration-profile adapter. Keep upstream's two-layer loader and trust gate intact.

use crate::config::{Config, ConfigLoadError, ConfigRaw};
use crate::keymap::merge_keys;
use helix_loader::{embedded::CONFIG, merge_toml_values};

pub(crate) fn read_config(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !CONFIG.is_empty() => {
            Ok(String::new())
        }
        result => result,
    }
}

pub(crate) fn load(
    global: Result<&String, ConfigLoadError>,
    local: Result<String, ConfigLoadError>,
) -> Result<Config, ConfigLoadError> {
    load_profile(CONFIG, global, local)
}

fn load_profile(
    embedded: &str,
    global: Result<&String, ConfigLoadError>,
    local: Result<String, ConfigLoadError>,
) -> Result<Config, ConfigLoadError> {
    if embedded.is_empty() {
        return Config::load(global, local);
    }
    // Merge sparse raw settings before deserializing editor defaults. Merging
    // fully defaulted Configs would erase profile fields omitted by later layers.
    let mut layers =
        vec![toml::from_str::<ConfigRaw>(embedded).map_err(ConfigLoadError::BadConfig)?];
    let global = global.and_then(|text| toml::from_str(text).map_err(ConfigLoadError::BadConfig));
    let local = local.and_then(|text| toml::from_str(&text).map_err(ConfigLoadError::BadConfig));
    match (global, local) {
        (Ok(global), Ok(local)) => layers.extend([global, local]),
        (_, Err(ConfigLoadError::BadConfig(error)))
        | (Err(ConfigLoadError::BadConfig(error)), _) => {
            return Err(ConfigLoadError::BadConfig(error))
        }
        (Ok(config), Err(_)) | (Err(_), Ok(config)) => layers.push(config),
        (Err(_), Err(_)) => {}
    }
    let mut config = Config::default();
    let mut editor = None;
    // Exhaustive matching makes upstream schema additions a compile error here
    // instead of silently dropping newly supported settings in profile builds.
    for ConfigRaw {
        theme,
        keys,
        editor: value,
    } in layers
    {
        config.theme = theme.or(config.theme);
        if let Some(keys) = keys {
            merge_keys(&mut config.keys, keys);
        }
        if let Some(value) = value {
            editor = Some(match editor {
                Some(previous) => merge_toml_values(previous, value, 3),
                None => value,
            });
        }
    }
    if let Some(editor) = editor {
        config.editor = editor.try_into().map_err(ConfigLoadError::BadConfig)?;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::KeyTrie;
    use helix_view::{document::Mode, theme};

    #[test]
    fn embedded_settings_are_defaults_for_external_and_local_settings() {
        let embedded = r#"
            theme = "embedded"
            [editor]
            scrolloff = 11
            line-number = "relative"
            [keys.normal]
            F9 = "move_line_up"
            F10 = "move_line_down"
            F11 = "move_prev_word_start"
        "#;
        let global = r#"
            theme = "external"
            [editor]
            scrolloff = 3
            [keys.normal]
            F9 = "move_next_word_start"
            F10 = "move_next_word_end"
        "#
        .to_owned();
        let local = r#"
            [editor]
            scrolloff = 7
            [keys.normal]
            F9 = "move_prev_word_end"
        "#
        .to_owned();
        let config = load_profile(embedded, Ok(&global), Ok(local)).unwrap();
        assert_eq!(
            config.theme,
            Some(theme::Config::Constant("external".into()))
        );
        assert_eq!(config.editor.scrolloff, 7);
        assert_eq!(
            config.editor.line_number,
            helix_view::editor::LineNumber::Relative
        );
        for (key, command) in [
            ("F9", "move_prev_word_end"),
            ("F10", "move_next_word_end"),
            ("F11", "move_prev_word_start"),
        ] {
            assert!(matches!(
                config.keys[&Mode::Normal].search(&[key.parse().unwrap()]),
                Some(KeyTrie::MappableCommand(cmd)) if cmd.name() == command
            ));
        }
        let config = load_profile(
            embedded,
            Err(ConfigLoadError::default()),
            Err(ConfigLoadError::default()),
        )
        .unwrap();
        assert_eq!(config.editor.scrolloff, 11);
        assert_eq!(
            config.theme,
            Some(theme::Config::Constant("embedded".into()))
        );
        assert!(load_profile(
            embedded,
            Ok(&"invalid = [".into()),
            Err(ConfigLoadError::default())
        )
        .is_err());
    }
}
