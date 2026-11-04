# Upstream attribution

## Source

| | |
|---|---|
| Project | Soroswap Core |
| Repository | https://github.com/soroswap/core |
| Commit | `6eade008ee6794075ad941609e0fef0d1124d3eb` |
| License | Apache License 2.0 (`LICENSE-APACHE` in this repository) |
| Original authors | Soroswap / PaltaLabs contributors (esteblock and others) |
| Design lineage | Uniswap V2 |

The following were imported into `contracts/`: `factory`,
`factory-interface`, `pair`, `router`, `library`, `token`.

Not imported: the TypeScript deployment tooling, the address API, the
`public/` address registry, the OtterSec audit PDF, per-crate lockfiles and
Makefiles, committed `.wasm` binaries, unused test helpers, and the
UniswapV2 Solidity reference sources (GPL-3.0).

## Licensing notes

- Derived contract manifests declare Apache-2.0 and Apache notices are
  preserved. The library declaration conflicts with upstream as described below;
  this review does not resolve its applicable licensing.
- New VELTRIXAE-only scripts and docs are under the repository's MIT
  license (`LICENSE`). The independent `tools/spec-tool` manifest declares
  Apache-2.0.
- Upstream's in-repo `contracts/library/Cargo.toml` declared
  `license = "GPL-3.0"` even though the repository license is Apache-2.0, and
  the router depended on `soroswap-library 0.3.0` from crates.io, which is
  published as GPL-3.0. VELTRIXAE no longer depends on the crates.io crate;
  it links the in-repository library source. **Obtain rightsholder/legal clarification
  of applicable licensing and distribution obligations before asserting a
  compliant release.**

## Modifications (relative to the upstream commit)

Behaviour changes:

1. `library::get_amount_in` returns `InsufficientLiquidity` when
   `amount_out >= reserve_out` (was a division by zero or a negative result).
2. Router swaps validate the path: >= 2 tokens, no repeated hop token, and
   each hop registered in the factory (`RouterPairDoesNotExist`).
3. LP token name/symbol: `<A>-<B> VELTRIXAE LP Token` / `<A>-<B>-VLX-LP`
   (was `Soroswap LP Token` / `-SOROSWAP-LP`).
4. `contractmeta` descriptions for factory, pair and router.
5. Factory/router `__constructor` entry points initialize configuration in the
   deployment transaction. Existing public `initialize` signatures/guards are
   retained. Constructor arguments are a deployment-interface change; existing
   immutable deployments are not migrated. See SECURITY.md and constructor tests.

6. Exact-output input quotes account for the pair's integer-rounded fee. The
   old continuous inverse plus one can underfund rare swaps by one input unit;
   sufficient historical quotes retain their exact value. Pair invariant, fee
   policy and LP formulas are unchanged. See `review/PR9_REMEDIATION.md` for
   the counterexample, derivation, tests and required independent math review.

Build and structure:

7. Single cargo workspace, soroban-sdk 20.2 / 22-rc -> 22.0.11, target
   `wasm32v1-none`, `overflow-checks = true` for every contract.
8. Crates renamed `soroswap-*` -> `veltrixae-*` (test token
   `soroban-token-contract` -> `veltrixae-test-token`).
9. Pair uses `soroban_sdk::token::TokenClient` instead of importing the
   test-token WASM.
10. Factory, router and library use trait-defined clients instead of
   `contractimport!` of compiled WASM (and a committed binary).
11. Library contract export gated behind the `contract` feature.
12. Factory uses `deploy_v2` (same address derivation).
13. Test-only contract methods moved into `#[cfg(test)]` impl blocks; two
    behaviour-preserving lint cleanups in pair storage.

Tests:

14. Upstream tests kept; wasm paths updated; two router event tests read
    events before the next invocation (SDK 21+ semantics); the two
    `should_panic` pair-does-not-exist tests now assert error 509.
15. New suites: factory security, pair boundaries, router security and path
    validation, library property tests.

Unchanged on purpose: existing callable function names and signatures, error enums and
codes, event topics and payloads, storage keys, pair fee policy and LP liquidity formulas,
pair address derivation.

Rust type names (`SoroswapFactory`, `SoroswapPairError`, ...) were kept so
that the contract spec and generated bindings stay compatible with
Soroswap-based consumers.

Licensing evidence and unresolved questions: [LICENSING_REVIEW.md](LICENSING_REVIEW.md).
