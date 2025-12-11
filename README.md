# pbcat

`pbcat` collects file contents and copies them to the system clipboard for quick sharing (e.g., with LLMs). It respects `.gitignore`, skips common build artifacts, and preserves deterministic ordering.

## Usage

```
pbcat [-s <separator>] [-H|--header] [--sort args|name] <file|dir> [more ...]
```

- `-s, --separator` — insert text between files.
- `-H, --header` — prefix each file with a header `== <path> ==`.
- `--sort args|name` — preserve argument order (default) or sort by path name.
- `--` — end option parsing.

Features:
- Accepts files and directories; walks directories recursively.
- Respects `.gitignore` (root and nested), git excludes, and common build dirs (e.g., `target`, `node_modules`, `DerivedData`).
- Deduplicates files seen via multiple paths.
- Prints a summary of files and bytes copied.

Clipboard utilities:
- macOS: `pbcopy`
- Linux: `wl-copy`, `xclip`, or `xsel`

## Development

- Run tests: `cargo test`
