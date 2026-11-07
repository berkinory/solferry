default:
    @just --list

fmt:
    cargo fmt --all
    taplo fmt

lint:
    cargo fmt --all --check
    taplo fmt --check
    typos
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo nextest run --workspace

deny:
    cargo deny check

check: lint test deny

build:
    cargo build --release -p solferry-cli

bundle-macos:
    cargo build --release -p solferry-desktop
    packaging/macos/bundle.sh target/release
