# VELTRIXAE contracts - developer tasks.
#
# Integration tests load the compiled WASM from target/wasm32v1-none/release,
# so `make test` always rebuilds the WASM first. Running `cargo test` on its
# own after changing contract code will test stale WASM.

WASM_DIR   := target/wasm32v1-none/release
CONTRACTS  := veltrixae_factory veltrixae_pair veltrixae_router
STELLAR    ?= stellar

.PHONY: all build optimize test test-only fmt fmt-check lint clean bindings metadata

all: test

## Compile every contract to WASM (wasm32v1-none, release profile).
build:
	cargo build --locked --target wasm32v1-none --release

## Shrink the deployable WASM with `stellar contract optimize`.
optimize: build
	@for c in $(CONTRACTS); do \
		$(STELLAR) contract optimize --wasm $(WASM_DIR)/$$c.wasm \
			--wasm-out $(WASM_DIR)/$$c.optimized.wasm || exit 1; \
	done

## Build the WASM, then run all unit, integration and property tests.
test: build
	cargo test --locked

## Run tests without rebuilding the WASM (only if the WASM is current).
test-only:
	cargo test --locked

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --locked --all-targets -- -A clippy::all -D clippy::correctness

## Generate TypeScript bindings for the frontend and aggregator.
bindings: build
	./scripts/generate-bindings.sh

## Export machine-readable contract interface metadata (JSON).
metadata: build
	./scripts/export-metadata.sh

clean:
	cargo clean
