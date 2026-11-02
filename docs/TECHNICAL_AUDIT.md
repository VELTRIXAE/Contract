# Technical audit

Verified findings from a code-level audit of this repository. Every claim
below was checked by running the command shown, reading the referenced
source, or diffing against the upstream commit in
[docs/UPSTREAM_ATTRIBUTION.md](UPSTREAM_ATTRIBUTION.md). Nothing here is
inferred from documentation alone.

Date: 2026-11-02. Commit audited: `7ea7885` (`main`).

## Scope

Six workspace crates (`Cargo.toml`):

| Crate | Path | Purpose |
|---|---|---|
| `veltrixae-factory` | `contracts/factory` | Pair registry, deployment, protocol-fee settings |
| `veltrixae-factory-interface` | `contracts/factory-interface` | Shared factory trait, client, error enum |
| `veltrixae-pair` | `contracts/pair` | One pool per token pair; reserves, swap, LP token |
| `veltrixae-router` | `contracts/router` | User-facing entry point; path validation, slippage, deadlines |
| `veltrixae-library` | `contracts/library` | Pure quote/reserve math, pair address derivation |
| `veltrixae-test-token` | `contracts/token` | Test-only SEP-41 token, not deployed |

## Build and test status (verified this session)

```
cargo build --target wasm32v1-none --release --locked   # exit 0, no warnings
cargo test --locked                                      # exit 0
cargo clippy --all-targets --locked -- -A clippy::all -D clippy::correctness   # exit 0
cargo fmt --all -- --check                                # exit 0 (after this audit's fix)
```

Test counts by crate, all passing, 0 failing:

| Crate | Unit | Property / integration | Doc |
|---|---|---|---|
| factory | 60 | - | - |
| factory-interface | 0 | - | 0 |
| library | 23 | 8 (`tests/amm_properties.rs`) | - |
| pair | 63 | - | - |
| router | 92 | - | - |
| test-token | 7 | - | - |
| **Total** | | | **253 passing, 0 failing** |

## Findings fixed in this audit

1. **No formatting enforcement, and most of the codebase was not
   rustfmt-compliant.** `cargo fmt --all -- --check` failed on 62 of 63
   source files (every contract crate except the already-clean test token) —
   693 diff hunks, all upstream Soroswap style never normalized. CI
   (`.github/workflows/ci.yml`) built, tested and ran clippy but never ran
   `cargo fmt --check`.
   **Fixed:** ran `cargo fmt --all` (formatting only, no behavioral change —
   confirmed by a full rebuild and test re-run before and after, same 253
   passing tests, identical WASM-affecting logic), and added a `Check
   formatting` step plus the `rustfmt` toolchain component to CI so this
   cannot regress silently again.

2. **Two live compiler warnings on a clean build.**
   - `contracts/library/src/reserves.rs`: `get_reserves_with_pair` destructured
     `sort_tokens` into `(token_0, token_1)` but never read `token_1`.
     **Fixed:** renamed to `_token_1` (the binding is dead by design — the
     pair address is already known, so only `token_0` is needed to decide
     which reserve is "a").
   - `contracts/pair/src/soroswap_pair_token/mod.rs`: `pub use
     contract::SoroswapPairTokenClient;` was unused outside `#[cfg(test)]`
     code (the module is private — `mod soroswap_pair_token;` in
     `contracts/pair/src/lib.rs` — so the re-export is invisible outside the
     crate, and inside the crate only test files use it). **Fixed:** gated
     the re-export behind `#[cfg(test)]`, consistent with the existing
     pattern described in
     [docs/UPSTREAM_ATTRIBUTION.md](UPSTREAM_ATTRIBUTION.md) item 11
     ("test-only contract methods moved into `#[cfg(test)]` impl blocks").

   Both are lint-only; neither affects any already-fixed security property
   or the compiled WASM behavior (`cfg(test)` code and an unused local
   binding are never part of a release build regardless).

## Findings documented, not changed

These were checked and confirmed accurate; no code change was warranted.

- **AMM math** (`contracts/library/src/quotes.rs`, pair `deposit`/`swap`/
  `withdraw`/`mint_fee` in `contracts/pair/src/lib.rs`) matches Uniswap V2 /
  upstream Soroswap exactly, including the already-applied
  `get_amount_in` division-by-zero fix
  (`amount_out >= reserve_out -> InsufficientLiquidity`, verified present at
  `contracts/library/src/quotes.rs:78`). Per `CONTRIBUTING.md`, this code is
  off-limits without a written justification and a second reviewer — none
  of this audit's findings justify touching it.
- **Deployment script safety guards** described in `docs/DEPLOYMENT.md`
  were checked against the actual scripts: `scripts/lib.sh` refuses any
  network name containing `mainnet`/`public`/`pubnet`
  (`scripts/lib.sh:33-34`), and `scripts/deploy-testnet.sh` requires either
  an interactive `DEPLOY TO TESTNET` confirmation or the explicit
  `--i-authorize-testnet-deploy` flag (`scripts/deploy-testnet.sh:66-68`).
  Documentation matches implementation.
- **No deployment has been made from this repository.** `deployments/`
  contains only `README.md`; `README.md` states this explicitly. Confirmed
  by directory listing, not just the claim in the README.
- **Dependency posture.** `soroban-sdk`/`soroban-token-sdk` pinned at
  `22.0.11` (latest 22.x patch at audit time). `.github/dependabot.yml`
  deliberately excludes semver-major bumps for `soroban-sdk`,
  `soroban-token-sdk`, `ed25519-dalek` (host-side major, incompatible with
  SDK 22) and the bindings' `typescript` (generated `tsconfig` doesn't build
  under TS7 yet), each with a one-line rationale in the config file itself.
  This is a deliberate, documented deferral, not neglect — appropriate per
  `CONTRIBUTING.md`'s rule to migrate SDK majors "deliberately on a branch".
- **`cargo-outdated` is not available in this environment** (not installed,
  and installing it was not attempted given the known resource constraints
  noted below); dependency currency was instead checked by reading
  `Cargo.lock` and `dependabot.yml` directly.

## Findings documented, deferred (require a decision this audit cannot make)

- **Library crate licensing.** `docs/UPSTREAM_ATTRIBUTION.md` already flags
  that upstream's `contracts/library` declared `GPL-3.0` despite the
  repository being Apache-2.0, and that VELTRIXAE now vendors the source
  instead of depending on the GPL-3.0 `soroswap-library` crate from
  crates.io. The existing note — "have the licensing of the library crate
  confirmed by counsel before a commercial release" — still stands. This
  audit did not change that assessment; it is a legal question, not a code
  one.
- **Deprecated soroban-sdk test helpers.** A clean `cargo test --locked`
  run emits roughly 110 deprecation warnings across factory, pair and
  router test suites: `Env::register_contract` / `register_contract_wasm`
  (-> `register`), `Env::register_stellar_asset_contract` (->
  `register_stellar_asset_contract_v2`), and `Env::budget()` (->
  `cost_estimate().budget()`). All are in `#[cfg(test)]` code only — they
  do not appear in a release build and do not fail CI's
  `-D clippy::correctness` gate. Mechanically replacing ~100 call sites
  across every test file was judged out of scope for this audit: it is a
  real but non-urgent modernization with no security or correctness
  benefit, and touching that much test code without a specific reason
  risks silently changing test semantics (`register` vs `register_contract`
  differ in which deprecated-until-removed behaviors they opt into). Worth
  a dedicated PR.

## Differences from upstream Soroswap

Already fully enumerated in
[docs/UPSTREAM_ATTRIBUTION.md](UPSTREAM_ATTRIBUTION.md); this audit
re-verified items 1 (division-by-zero fix), 2 (router path validation) and
5 (single release profile with `overflow-checks = true`) by reading the
current source, not just trusting the document. All three are present and
match the description.

## What this audit did not re-litigate

This repository already carries a documented internal security review
(`SECURITY.md`, "Review summary (VELTRIXAE migration)" table) covering
quote-math division-by-zero, router path validation, integer overflow
checks, router WASM surface, build reproducibility, authorization and
reserve invariants. This audit re-verified the specific claims above rather
than repeating that review from scratch, per the standing instruction in
`SECURITY.md` itself: "not a substitute for an independent audit."
