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

- Files derived from Soroswap Core remain under Apache-2.0. Their crate
  manifests declare `license = "Apache-2.0"`.
- New VELTRIXAE-only files (scripts, docs, tools) are under the repository's
  MIT license (`LICENSE`).
- Upstream's in-repo `contracts/library/Cargo.toml` declared
  `license = "GPL-3.0"` even though the repository license is Apache-2.0, and
  the router depended on `soroswap-library 0.3.0` from crates.io, which is
  published as GPL-3.0. VELTRIXAE no longer depends on the crates.io crate;
  it links the in-repository library source. **Have the licensing of the
  library crate confirmed by counsel before a commercial release.**

## Modifications (relative to the upstream commit)

Behaviour changes:

1. `library::get_amount_in` returns `InsufficientLiquidity` when
   `amount_out >= reserve_out` (was a division by zero or a negative result).
2. Router swaps validate the path: >= 2 tokens, no repeated hop token, and
   each hop registered in the factory (`RouterPairDoesNotExist`).
3. LP token name/symbol: `<A>-<B> VELTRIXAE LP Token` / `<A>-<B>-VLX-LP`
   (was `Soroswap LP Token` / `-SOROSWAP-LP`).
4. `contractmeta` descriptions for factory, pair and router.

Build and structure:

5. Single cargo workspace, soroban-sdk 20.2 / 22-rc -> 22.0.11, target
   `wasm32v1-none`, `overflow-checks = true` for every contract.
6. Crates renamed `soroswap-*` -> `veltrixae-*` (test token
   `soroban-token-contract` -> `veltrixae-test-token`).
7. Pair uses `soroban_sdk::token::TokenClient` instead of importing the
   test-token WASM.
8. Factory, router and library use trait-defined clients instead of
   `contractimport!` of compiled WASM (and a committed binary).
9. Library contract export gated behind the `contract` feature.
10. Factory uses `deploy_v2` (same address derivation).
11. Test-only contract methods moved into `#[cfg(test)]` impl blocks; two
    behaviour-preserving lint cleanups in pair storage.

Tests:

12. Upstream tests kept; wasm paths updated; two router event tests read
    events before the next invocation (SDK 21+ semantics); the two
    `should_panic` pair-does-not-exist tests now assert error 509.
13. New suites: factory security, pair boundaries, router security and path
    validation, library property tests.

Unchanged on purpose: all function names and signatures, error enums and
codes, event topics and payloads, storage keys, fee and liquidity math,
pair address derivation.

Rust type names (`SoroswapFactory`, `SoroswapPairError`, ...) were kept so
that the contract spec and generated bindings stay compatible with
Soroswap-based consumers.
