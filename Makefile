.PHONY: all fmt lint test test-default test-dates test-timezones test-wasm build-wasm web-test check package clean

all: check

fmt:
	cargo fmt --all

lint:
	cargo fmt --all --check
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features

# The build most crates.io users get.
test-default:
	cargo test

test-dates:
	cargo test --features dates-jiff

# Exercised on its own because code reachable only here has shipped bugs before.
test-timezones:
	cargo test --features timezones-jiff

build-wasm:
	wasm-pack build --target web --out-dir pkg -- --features wasm
	wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm

test-wasm: build-wasm
	node tests/wasm_node_smoke.mjs
	node tests/web_adapters.mjs
	node tests/react_adapter_runtime.mjs

web-test:
	cd web && npm run test:types

package:
	cargo publish --dry-run

check: lint test test-default test-dates test-timezones

clean:
	cargo clean
	rm -rf pkg pkg-node dist
