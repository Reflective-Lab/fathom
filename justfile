default:
    @just --list

build:
    cargo build --workspace

check:
    cargo check --workspace --all-targets

test:
    cargo test --workspace

lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check

fmt:
    cargo fmt --all

up:
    docker compose up -d

down:
    docker compose down

logs:
    docker compose logs -f
