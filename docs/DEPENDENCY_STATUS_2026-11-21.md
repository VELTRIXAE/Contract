# Dependency status — 2026-11-21

**RUSTSEC-2026-0387** (`crypto-bigint 0.5.5`, patched in `>=0.7.6`) is
resolved. The advisory concerns timing variability in operations intended to be
constant-time. See the
[RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html).

### Original finding

`cargo tree --locked --offline -i crypto-bigint` showed the path:
`soroban-sdk 22.0.11 -> soroban-env-host 22.1.3 -> elliptic-curve 0.13.8 -> crypto-bigint 0.5.5`,
also through the host's ECDSA/k256/p256 dependencies.
`cargo tree --locked --target wasm32v1-none -i crypto-bigint` reported nothing
to print, scoping the finding to the local host/test dependency graph rather
than the contract's WASM target graph. The installed elliptic-curve 0.13
manifest required crypto-bigint 0.5, so a direct lockfile bump to 0.7.6 was
incompatible.

### Resolution

The coupled SDK/host/curve dependencies were migrated together to an
upstream-reviewed compatible set in which the host's elliptic-curve dependency
resolves crypto-bigint to a patched release (`>=0.7.6`). No advisory ignore,
fail-open flag or security policy exception was added; the fix is a real
dependency upgrade, not a suppression. Contract interfaces are unchanged.

`cargo tree --locked --offline -i crypto-bigint` now reports a patched version
on the native host/test path, and
`cargo tree --locked --target wasm32v1-none -i crypto-bigint` still reports
nothing to print.

Verification performed:

- All contract, auth and rounding test suites pass.
- Bindings and metadata were regenerated and the checked-in outputs match.
- Both `Cargo.lock` and `tools/spec-tool/Cargo.lock` pass strict
  `cargo audit` with no advisory ignores.

This closes the advisory finding only. Licensing, the funded Testnet pool and
wallet-signed swap evidence remain independent release blockers; do not claim
production readiness on the basis of this change.

## Generator repair

Merged dependency updates had left duplicate `stellar-xdr 30.0.0` entries in
`tools/spec-tool/Cargo.lock`, while the manifest still pinned the TypeScript
generator to 28.1.0. Restore a compatible parser/XDR 28.0.0 pair and regenerate
the lockfile with Rust 1.99.1. The generator compiles with `--locked`.
Dependabot groups these three coupled crates and defers major upgrades to a
reviewed migration that regenerates the checked-in clients and metadata.
This change does not alter AMM contract code or the root dependency lockfile.

Reproduce:

```sh
cargo build --locked --manifest-path tools/spec-tool/Cargo.toml
make bindings metadata
git diff --exit-code -- bindings/ metadata/
cargo audit --file Cargo.lock
cargo audit --file tools/spec-tool/Cargo.lock
```

The generator lockfile repair is separate from the RUSTSEC-2026-0387 resolution
above. Build/test, generated-output comparison and advisory jobs must be
reported separately.
