run *args:
    cargo run -- {{args}}

run-ai *args:
    AGENT=1 cargo run -- {{args}}

build:
    cargo build --release
    mkdir -p bin
    cp target/release/dashlane-to-bitwarden bin/dashlane-to-bitwarden

test:
    cargo test --all-targets

lint:
    rustfmt --check --edition 2024 --config skip_children=true src/lib.rs src/main.rs src/input.rs src/model.rs src/convert.rs tests/behavior.rs
    cargo clippy --all-targets -- -D warnings

coverage:
    cargo llvm-cov --all-targets --fail-under-lines 90

check: lint test coverage
