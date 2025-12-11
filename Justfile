default:
    just --list

verify:
    cargo fmt -- --check

test:
    cargo test

clippy:
    cargo clippy --all-targets -- -D warnings

install:
    cargo install --path . --locked

ci:
    cargo fmt -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    export PBCAT_CLIPBOARD_FILE="$(pwd)/pbcat-smoke.out" && \
    echo "pbcat smoke" > pbcat-smoke.txt && \
    cargo run -- pbcat-smoke.txt && \
    cat "$PBCAT_CLIPBOARD_FILE"
