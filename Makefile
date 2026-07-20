.PHONY: all fmt lint test test-dates test-wasm build-wasm web-test check package clean

all: check

fmt:
	cargo fmt --all

lint:
	cargo fmt --all --check
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features

test-dates:
	cargo test --features dates-jiff

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

check: lint test test-dates

clean:
	cargo clean
	rm -rf pkg pkg-node dist
