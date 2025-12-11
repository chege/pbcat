default: test

verify:
    cargo fmt -- --check

test:
    cargo test
