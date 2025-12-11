# pbcat

`pbcat` is a fast CLI that gathers file contents and sends them to your clipboard for frictionless sharing (e.g., with LLMs). It respects `.gitignore`, skips build artifacts, dedupes inputs, and preserves deterministic ordering.

## Quickstart

```
pbcat [-s <separator>] [-H|--header] [--no-header] [--sort args|name] [-L|--list] <file|dir> [...]
```

- `-s, --separator` — insert text between files.
- `-H, --header` — force headers for each file.
- `--no-header` — disable headers (by default headers are auto-enabled when copying multiple files).
- `--sort args|name` — preserve argument order (default) or sort by path name.
- `-L, --list` — dry-run: list selected files and total bytes; do not touch the clipboard.
- `--` — end option parsing.

Examples:

```
# Copy a few files with headers and blank separator
pbcat -H file1.rs file2.rs

# Copy a directory tree with separators, sorted by name
pbcat --sort name -s "\n---\n" src/

# Dry-run to see what would be copied
pbcat -L src/ README.md
```

## Features

- Accepts files and directories; walks directories recursively.
- Respects `.gitignore` (root and nested), git excludes, and common build dirs (e.g., `target`, `node_modules`, `DerivedData`).
- Deduplicates files seen via multiple paths.
- Prints a summary of files and bytes copied.

Clipboard utilities:
- macOS: `pbcopy`
- Linux: `wl-copy`, `xclip`, or `xsel`
- Windows: `clip` or `powershell -Command Set-Clipboard`
- Tests/dev: set `PBCAT_CLIPBOARD_FILE=/tmp/pbcat.out` to write there instead of the clipboard.

## Install

Quick install (macOS/Linux):

```
curl -fsSL https://raw.githubusercontent.com/chege/pbcat/main/scripts/install.sh | sh
```

`INSTALL_DIR` is optional; defaults to `$HOME/.local/bin` (user-writable). Ensure it's on your PATH. Artifacts are pulled from the latest GitHub release.

If the download fails or a release asset is unavailable, install from source instead:

```
cargo install --git https://github.com/chege/pbcat.git --locked
```

Install from source into Cargo’s global bin dir (`~/.cargo/bin`):

```
cargo install --path . --locked
```

Ensure `~/.cargo/bin` is on `PATH` (zsh):

```
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Run the binary:

```
pbcat --help
```

Build without installing:

```
cargo build --release
ln -sf "$(pwd)/target/release/pbcat" /usr/local/bin/pbcat
```

## Clipboard backends
- macOS: `pbcopy`
- Linux: `wl-copy`, `xclip`, or `xsel`
- Tests/dev: set `PBCAT_CLIPBOARD_FILE=/tmp/pbcat.out` to write there instead of the clipboard.

## Helpful `just` recipes

- `just` — list tasks.
- `just verify` — `cargo fmt -- --check`
- `just test` — `cargo test`
- `just clippy` — `cargo clippy --all-targets -- -D warnings`
- `just install` — `cargo install --path . --locked`

## Development

- Run tests: `cargo test`
