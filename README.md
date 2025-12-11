# pbcat

`pbcat` collects file contents and copies them to the system clipboard for quick sharing (e.g., with LLMs). It respects `.gitignore`, skips common build artifacts, and preserves deterministic ordering.

## Usage

```
pbcat [-s <separator>] [-H|--header] [--sort args|name] [-L|--list] <file|dir> [more ...]
```

- `-s, --separator` — insert text between files.
- `-H, --header` — prefix each file with a header `== <path> ==`.
- `--sort args|name` — preserve argument order (default) or sort by path name.
- `-L, --list` — dry-run: list selected files and total bytes; do not touch the clipboard.
- `--` — end option parsing.

Features:
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

Install into Cargo’s global bin dir (`~/.cargo/bin`):

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

## Helpful `just` recipes

- `just` — list tasks.
- `just verify` — `cargo fmt -- --check`
- `just test` — `cargo test`
- `just clippy` — `cargo clippy --all-targets -- -D warnings`

## Development

- Run tests: `cargo test`
