# Vero Core Contracts — Makefile
#
# Targets:
#   build       — Compile the WASM contract
#   test        — Run all unit and integration tests
#   check       — Quick syntax check without full compilation
#   verify      — Run Kani formal verification harnesses
#   invariants  — Run runtime safety invariant tests
#   proofs      — Run K-framework proofs (requires K Framework)
#   all         — Build, test, and verify

.PHONY: build test check verify invariants proofs all

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

check:
	cargo check --features verification

# Kani proof harnesses (requires cargo-kani installed)
verify:
	cargo kani --manifest-path verification/Cargo.toml

# Runtime invariant tests (pure consensus logic, no Soroban host)
invariants:
	cargo test --test safety_invariants --features verification

# K-framework proofs (requires K Framework 6.0+)
proofs:
	cd proofs && bash build.sh

all: check test invariants verify