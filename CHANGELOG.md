# Changelog

All notable changes to this repository are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project does
not yet cut versioned releases (no verified deployment records are available — see
`deployments/README.md`), so entries accumulate under **Unreleased** until
a first testnet deployment is tagged.

## Unreleased

### Added
- `docs/TECHNICAL_AUDIT.md`: verified build/test status, formatting
  compliance, and outstanding dependency/licensing items.
- CI now runs `cargo fmt --all -- --check` (and installs the `rustfmt`
  toolchain component) so formatting regressions fail the build.

### Fixed
- PR #10: exact-output quotes now cover integer fee rounding without lowering
  sufficient historical quotes. Includes a deterministic counterexample,
  persisted property seed and release-WASM swap regression. Merged 2026-11-04
  with no recorded independent math review; that review is still required.
- Deployment verifier now fails unavailable or malformed factory pair counts.
- Applied `cargo fmt --all` across every contract crate; the codebase had
  never been reformatted from its original upstream Soroswap style (693
  diff hunks, formatting only — no behavioral change, re-verified by a full
  build and 253-test re-run before and after).
- `veltrixae-library`: removed a dead-code warning on the unused `token_1`
  binding in `get_reserves_with_pair`.
- `veltrixae-pair`: gated the test-only `SoroswapPairTokenClient` re-export
  behind `#[cfg(test)]`, removing an unused-import warning on a clean
  (non-test) build.

### Known limitations (carried forward, not new)
- No contracts have been deployed from this repository yet.
- `contracts/library`'s licensing (Apache-2.0 repository, GPL-3.0 upstream
  declaration) needs counsel sign-off before a commercial release — see
  `docs/UPSTREAM_ATTRIBUTION.md`.
- soroban-sdk test helpers (`register_contract`, `register_stellar_asset_contract`,
  `Env::budget()`) are deprecated upstream; ~100 call sites in test code
  still use them. Test-only, does not affect release builds. Tracked in
  `docs/TECHNICAL_AUDIT.md`.
