.PHONY: version-check fmt lint build lint-for-production test

version-check:
	rustup --version # rust version manager
	rustc --version # compiler
	cargo --version # package manager

build:
	cargo build

run: fmt lint build
	./target/debug/rust-axum-redis-example

fmt:
	cargo fmt

# clippy rules: https://rust-lang.github.io/rust-clippy/master/index.html
lint:
	cargo +nightly clippy -- -A clippy::print_literal

lint-for-production:
	cargo clippy -- -D warnings

fix:
	cargo fix
	cargo +nightly clippy --fix -Z unstable-options

test:
	cargo test

release: fmt lint test build
	cargo build --release

post:
	@curl -X POST localhost:3000/metric -H "Content-Type: application/json" -d $$(./scripts/generate_datum.sh)
