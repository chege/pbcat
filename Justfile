default:
    just --list

verify:
    cargo fmt -- --check

test:
    cargo test
