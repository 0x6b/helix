//! Prepare the complete standard grammar set without build-machine user settings.

use super::{build_grammar, ensure_git_is_available, fetch_grammar, run_parallel, Configuration};
use anyhow::{Context, Result};

pub fn prepare(target: &str, build: bool) -> Result<Vec<String>> {
    let config: Configuration = crate::config::default_lang_config().try_into()?;
    let names = config
        .grammar
        .iter()
        .map(|g| g.grammar_id.clone())
        .collect();
    if !build {
        return Ok(names);
    }
    ensure_git_is_available()?;
    // Use separate fetch/build passes, like the normal grammar commands. Parsing
    // the small configuration twice avoids changing upstream grammar types to Clone.
    for (name, result) in run_parallel(config.grammar, fetch_grammar) {
        result.with_context(|| format!("Fetching bundled grammar {name}"))?;
    }
    let config: Configuration = crate::config::default_lang_config().try_into()?;
    let target = target.to_owned();
    for (name, result) in run_parallel(config.grammar, move |grammar| {
        build_grammar(grammar, Some(&target))
    }) {
        result.with_context(|| format!("Building bundled grammar {name}"))?;
    }
    Ok(names)
}
