# Standalone builds and embedded configuration

This fork builds a standalone `hx`: copy `target/release/hx` after running
`cargo build --release`. No accompanying `runtime/` directory or user configuration
is needed for editing and syntax highlighting, including Japanese Markdown and
fenced code blocks. The fork's Japanese word-motion behavior is preserved.

The binary bundles all pinned grammars from the repository's `languages.toml`,
all runtime queries and themes, and the tutor. Default language settings and the
two default themes are also compiled in. Build-time user/workspace language
configuration does not change the bundled grammar set. Building requires Git,
C/C++ compilers, and network access to fetch the pinned grammar sources, in
addition to Rust. For an offline rebuild, set `HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1`
after preparing **all** built-in grammars; missing libraries fail the build rather
than producing an incomplete binary.

On first use, the bundle is unpacked into the user's cache directory
(`$XDG_CACHE_HOME/helix/runtime/<sha256>` on Linux, defaulting to
`~/.cache/helix/runtime/<sha256>`), used as the lowest-priority runtime fallback.
Subsequent processes reuse it; a changed bundle gets a separate cache directory.
The archive's SHA-256 is computed at build time and embedded in the executable;
startup uses that value without hashing the archive again.
Extraction is published atomically so concurrent startups cannot see partial
contents. Existing user themes, `languages.toml`, runtime directories (including
`HELIX_RUNTIME`), and workspace trust rules retain their existing behavior.
Theme inheritance and query inheritance work across external and bundled files.
The cache is trusted like other per-user runtime files: do not share it with
untrusted users. A writable cache filesystem that permits loading native libraries
is required on first use. Old bundle caches are not automatically removed because
other editor processes may still use them; they can be deleted when no editor is
using them and are recreated on next use. This is single-file distribution, not
execution without extraction.

Language servers, formatters, clipboard utilities, terminal fonts, and OS shared
libraries are not bundled. They remain optional external tools; none is needed
for basic Japanese Markdown editing or Tree-sitter highlighting. Build a binary
for each target OS/architecture; this does not make one executable portable
between different operating systems or incompatible libc versions.

## Embedding a configuration profile

Set `HELIX_CONFIG_DIR` to an **absolute** directory path when building to include
your own settings. For example, a parent `helix-config` repository can keep its
settings in `config/` and this repository as a `helix/` submodule:

```sh
HELIX_CONFIG_DIR="$PWD/config" cargo build --locked --release --manifest-path helix/Cargo.toml
```

The directory may contain `config.toml`, `languages.toml`, and `themes/*.toml`;
each is optional. Only these settings and themes are included, not other files
from the directory. Without `HELIX_CONFIG_DIR`, builds retain the normal defaults.
The path is only used at build time and is not needed on the destination machine.
Changes to the directory or environment variable trigger a rebuild. Invalid TOML
syntax fails the build; setting names and values are checked by the normal
runtime configuration loader.

Settings merge in this order: Helix defaults, embedded profile, external user
settings (or the file selected with `-c`), then trusted workspace settings.
External settings override only the fields they specify, including keybindings.
Workspace settings cannot override the effective global workspace-trust policy.
Embedded themes take precedence over bundled standard themes, but external themes
still win. Same-name theme inheritance works across these layers.

The two TOML files are read directly from the executable, not written into the
cache or the user's configuration directory. Profile themes use the runtime
cache. This does not bundle new grammar libraries named in a profile, nor tools
referenced by settings; the complete standard grammar set remains unchanged.
Do not embed credentials or machine-specific paths: the settings are recoverable
from the binary and are distributed with it.

## Upstream integration boundaries

Standalone-specific implementation and tests live in dedicated `embedded` modules:

- `helix-loader/build/embedded.rs`: build-time profile inputs.
- `helix-loader/src/embedded.rs`: embedded data and runtime cache.
- `helix-loader/src/grammar/embedded.rs`: complete standard grammar preparation.
- `helix-term/build/embedded.rs`: runtime archive packaging.
- `helix-term/src/embedded.rs`: profile configuration adapter.

Existing files only declare modules or call their entry points. The original
`Config::load` interface and implementation are unchanged; builds without a
profile use it directly. Profile merging is separate so adding a third settings
layer does not rewrite that function. The existing workspace-trust gate still
decides whether local settings can be passed to the adapter.

This layout reduces textual merge conflicts, not all maintenance. The adapter
uses upstream's configuration types and keymap merge operation; changes to their
fields or semantics may require updating the adapter. It matches configuration
fields exhaustively, so additions to the raw schema fail compilation instead of
silently being ignored by profile builds. Grammar preparation is a child module
so it can reuse private fetch/build operations without modifying
their signatures or adding `Clone` to upstream types. Those internal APIs also
remain integration points to check after an upstream update.
