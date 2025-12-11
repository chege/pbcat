default:
    just --list

verify:
    cargo fmt -- --check

test:
    cargo test

clippy:
    cargo clippy --all-targets -- -D warnings
