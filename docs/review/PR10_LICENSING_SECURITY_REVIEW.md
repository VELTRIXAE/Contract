# PR #10 follow-up: licensing matrix and RustSec remediation

Reviewed 2026-11-04 against `main` at `9ce68dea0dad08bd05efc67f50160521bead29a7`.
This is an engineering provenance review, not a legal opinion or an independent
audit. It changes documentation only: no contract, manifest, lockfile, license
text, notice, CI gate or deployment state was changed.

## Summary

| Item | Status |
|---|---|
| Licensing conflict in the 12 library files and the linked router | **OPEN.** Needs a rights-holder or legal decision |
| Properly licensed upstream version of the library | **NONE FOUND.** All 16 published crate versions and all 43 upstream refs declare GPL-3.0 |
| Ready-made permissive replacement | **NONE FOUND.** Candidates are building blocks only |
| Uniswap V2 lineage (GPL-3.0) of router, pair and factory | **NEW QUESTION.** May widen the scope beyond the library; for counsel |
| RUSTSEC-2026-0387 | **OPEN.** No compatible upgrade exists through `soroban-sdk 30.0.0-rc.1`; gate stays enabled and failing |
| Exact-output quote correction | **ALREADY MERGED** without a recorded independent review; review still needed |
| Testnet deployment | **NOT VERIFIED.** Nothing deployed, initialized or submitted |

## 1. Merge state of PR #10

The request for this review said the mathematical correction must not merge
until independently reviewed. GitHub records that it had already merged:

- PR #10 was marked ready for review at 2026-11-04T16:15:30Z and merged at
  16:15:40Z by `Novastrum1`, as merge commit `9ce68de`.
- The PR has zero recorded reviews.
- Its RustSec job had concluded `FAILURE`, and the post-merge `main` run
  (41865442618) also failed. Because of the deliberately retained advisory, the
  same applied to PR #9.
- Branch-protection settings could not be read with this token (HTTP 403), so
  whether that job is required or was bypassed is **NOT VERIFIED**.

So `main` now contains the correction to `contracts/library/src/quotes.rs`, and
the router WASM built from `main` includes it. This review did not revert or
rewrite anything. See [section 6](#6-decisions-for-maintainers).

## 2. Upstream provenance

Source commit: [`soroswap/core@6eade00`](https://github.com/soroswap/core/tree/6eade008ee6794075ad941609e0fef0d1124d3eb),
which was still the tip of upstream `main` when checked. That commit carries four
different license signals:

| Location at that commit | Declaration |
|---|---|
| `LICENSE` | Apache-2.0; its appendix reads `Copyright 2022 Stellar Development Foundation` |
| `package.json:15` | MIT |
| `contracts/library/Cargo.toml:9`, `contracts/library/README.md:51` | GPL-3.0 |
| `contracts/factory/UniswapV2/LICENSE` | GPL-3.0 full text, beside Uniswap Solidity reference files |

Upstream's own `docs/modules/library.md:73` says the library is GPL-3.0, "unlike
the repo root MIT license". The router, pair, factory, factory-interface and token
manifests upstream declare no license at all.

**Rights holders: not established.** No imported library or router file contains a
copyright notice. Git authorship is not copyright ownership, so the following is
only a list of whom to ask:

- `esteblock` is the sole recorded Git author of all 12 library files and the sole
  crates.io owner of `soroswap-library`.
- The manifest address is `esteblock@paltalabs.io`, and the Soroswap organization
  README says the project is developed by the PaltaLabs team. Whether an employer
  or organization owns the copyright is unknown.
- `git log --follow` on `contracts/library/Cargo.toml` also lists Stellar authors.
  That is a rename-detection artefact, reaching back to an unrelated root manifest.
  Without `--follow` the only author is `esteblock`, and the file first appears in
  `588232d`.

## 3. File-by-file matrix: the 12 library files

All rows share the same license evidence (crate GPL-3.0, README GPL-3.0, no file
header, root Apache-2.0) and the same blocker: **no file may be relabelled until a
grant or a legal decision exists.** "Local delta" is lines added/removed relative to
upstream at `9ce68de`. It includes formatting and renames, so it does not measure
independence from upstream.

| # | Local file under `contracts/library/` | Upstream added / last changed | Local delta | Uniswap V2 analogue | Replacement option if rights cannot be cleared |
|---|---|---|---|---|---|
| 1 | `Cargo.toml` | `588232d` 2023-10-05 / `fdc28f6` 2024-12-18 | +17/-29 | none | Metadata. Decide the `license` value only after the outcome; today it inherits Apache-2.0 and conflicts with row 2 |
| 2 | `README.md` | `3ff6970` 2023-10-13 / `284a5e1` 2024-02-06 | +2/-2 | Names `UniswapV2Library.sol` as its inspiration | Keep the GPL and acknowledgement text unchanged until resolved |
| 3 | `src/error.rs` | `1dc8cc9` 2023-12-11 / `2974844` 2023-12-11 | +2/-2 | Revert strings | New enum. Codes 301-306 are client-visible and must keep their values |
| 4 | `src/lib.rs` | `588232d` 2023-10-05 / `fdc28f6` 2024-12-18 | +115/-62 | Module layout | New wrapper. The stand-alone contract sits behind the `contract` feature, is off by default, and the router does not use it |
| 5 | `src/math.rs` | `d95c771` 2024-02-06 / same | +1/-1 | none | New checked ceiling division. `num-integer` (MIT OR Apache-2.0) is already a dependency |
| 6 | `src/quotes.rs` | `759b338` 2023-10-16 / `fdc28f6` 2024-12-18 | +82/-16 | `quote`, `getAmountOut`, `getAmountIn`, `getAmountsOut`, `getAmountsIn` | **Highest risk.** Rewrite from the specification, including the fee rounding, then test differentially against the current code |
| 7 | `src/reserves.rs` | `788bf15` 2023-10-16 / `fdc28f6` 2024-12-18 | +38/-23 | `getReserves` | New thin cross-contract call over the pair interface |
| 8 | `src/tokens.rs` | `e235daa` 2023-10-16 / `11021fe` 2024-07-25 | +12/-5 | `sortTokens`, `pairFor` | New implementation over the SDK deployer API. The salt scheme must stay byte-identical or pair addresses change |
| 9 | `src/test.rs` | `588232d` 2023-10-05 / `25e7574` 2024-12-17 | +22/-25 | none | Replace with newly written tests |
| 10 | `src/test/get.rs` | `e2351b4` 2023-12-11 / `fdc28f6` 2024-12-18 | +108/-50 | none | Replace with newly written tests |
| 11 | `src/test/quote.rs` | `e2351b4` 2023-12-11 / `64d630a` 2023-12-11 | +3/-3 | none | Replace with newly written tests |
| 12 | `src/test/tokens.rs` | `f96c53c` 2024-02-06 / `1e3bf7f` 2024-02-06 | +39/-13 | none | Replace with newly written tests |

Rows 9-12 are not linked into release WASM, but they are distributed in this source
repository, so they stay in scope. `tests/amm_properties.rs` is VELTRIXAE test work,
not an imported file. It exercises the library and settles nothing about it.

### Linked router

`veltrixae-router` depends on `veltrixae-library` as an `rlib`, so the library's code
is compiled into router WASM. There are 21 references in production source
(`src/lib.rs` 20, `src/error.rs` 1) using `get_reserves_with_factory`, `quote`,
`sort_tokens`, `pair_for`, `get_amount_out`, `get_amount_in`, `get_amounts_out`,
`get_amounts_in` and `SoroswapLibraryError`. `SoroswapLibraryError` is part of the
router's public error surface.

| Router material | Declared license | Issue |
|---|---|---|
| `contracts/router/src/lib.rs`, `error.rs` | Local Apache-2.0; none upstream | Direct library link. Also has the Uniswap question in section 4 |
| `event.rs`, `factory.rs`, `pair.rs`, `storage.rs` | Local Apache-2.0; none upstream | Compiled into the same WASM |
| `src/test/*` | Local Apache-2.0; none upstream | Not in WASM; no direct library references |
| Router release WASM | None of its own | Combined artifact; which terms apply is the central legal question |

Upstream linked the published GPL `soroswap-library 0.3.0` here. The current
`Cargo.lock` has no registry copy, but replacing the crate with vendored source does
not change where the code came from.

## 4. New question: Uniswap V2 lineage

Earlier reviews scoped the conflict to the library and treated factory and pair as
outside it, because they have no dependency edge to the library. That holds for the
library link. It does not address a separate question the upstream history raises:

| Evidence | Detail |
|---|---|
| Uniswap licenses | `Uniswap/v2-core` and `Uniswap/v2-periphery` both report GPL-3.0 |
| Pair | Upstream `c012e66` (2023-04-27): "copy the liquidity_pool contract, copy the UniswapV2Pair contract and comment the liquidity_pool contract to see what to change and what to keep". It adds `UniswapV2Pair.sol` and `UniswapV2ERC20.sol`. Local `contracts/pair/README.md` is titled "Pair: UniswapV2 Inspiration" |
| Router | Upstream `fce138e` (2023-07-19): "feat(router): add UniswapV2 inspiration"; `UniswapV2Router02.sol` was held in the tree until `d891832` (2023-12-12) |
| Library | Upstream `588232d` adds `UniswapV2Library.sol` in the same commit as the Rust crate. All eight of its functions have a same-named Rust counterpart, with `getReserves` split into two |
| Factory | Upstream still carries `UniswapV2Factory.sol` and the GPL text at the imported commit |
| Local statement | `NOTICE` says Soroswap Core "is itself derived in design from Uniswap V2" and describes the imported code as Apache-2.0 |

The Solidity files were not imported here, and matching an algorithm or function
names does not by itself make something a derivative work. **This review reaches no
such conclusion.** The point is scope: counsel should be asked about all four
Uniswap-lineage crates, not only the library. It also affects replacement, because
a rewrite modelled on Uniswap V2 could raise the same question.

## 5. Replacement candidates

| Candidate | License (GitHub metadata) | Usable? |
|---|---|---|
| `soroswap-library`, all 16 crates.io versions | GPL-3.0 | Not a permissive source |
| Soroswap Core, all 43 refs | Library declares GPL-3.0 on every ref | Not a permissive source |
| `stellar/soroban-examples` `liquidity_pool` | Apache-2.0 | Different single-pool design; not API-compatible. Possible clean starting point for pool logic only |
| `paltalabs/deterministic-address-soroban` | MIT | A playground for deterministic addresses, cited by `pair_for`; not a library |
| `OpenZeppelin/stellar-contracts` | MIT | No AMM package or `get_amount_out` found |
| `Phoenix-Protocol-Group/phoenix-contracts` | GPL-3.0 | Not permissive, and a different design |
| `CometDEX/comet-contracts-v1` | AGPL-3.0 | Not permissive, and a different design |

No drop-in permissive replacement was found. A rewrite is the only engineering route,
and its limit is process rather than code:

- **Authors:** the code would need authors with documented separation from this
  source. Everyone who has worked in this repository has read it, including the AI
  assistants used for this review, so they should not be treated as independent
  authors. Counsel should set the process.
- **Bar to replace:** the same public functions, error codes 301-306 and pair-salt
  bytes; differential testing against the current quotes; the existing property
  suite; a new router WASM hash and fresh review.

## 6. Decisions for maintainers

**Licensing.** One of these, chosen with counsel:

1. Get a written grant or clarification from the rights holders.
2. Distribute the affected work under GPL-3.0, after counsel sets the scope and the
   source, notice and packaging obligations. This is the option that needs no rewrite.
3. Commission a process-controlled rewrite as described in section 5.
4. Withhold distribution and release claims for affected artifacts until resolved.

Until then, make no unqualified "Apache-2.0" or "MIT" claim for the library, the
router WASM or the project as a whole, including in grant material.

A rights-holder request should cover:

- which license governs `contracts/library` at `6eade00` and at the crate versions;
- whether the requester has authority over all contributions, and any employer or
  organization ownership;
- whether modified copies may be distributed under Apache-2.0, and which notices to
  keep;
- how the router, pair and factory are licensed, given that their manifests declare
  nothing and `package.json` says MIT;
- how upstream itself treats the GPL Uniswap V2 lineage.

**Math correction.** Commission the independent review now that it is on `main`. If
it fails, revert through a new PR. Do not rewrite history. No deployment should use
the corrected quote until then, and existing test evidence should not be described
as independent review.

**RustSec.** See section 7.

## 7. RUSTSEC-2026-0387: upgrade investigation

The [advisory](https://github.com/rustsec/advisory-db/blob/main/crates/crypto-bigint/RUSTSEC-2026-0387.md)
gives one patched range, `>= 0.7.6`, with no 0.5.x backport. `main` CI reports the
same result for `crypto-bigint 0.5.5`.

Each attempt below ran in a throwaway copy of `9ce68de`, so the repository's manifests
and lockfile are unchanged:

| Attempt | Result |
|---|---|
| `cargo update -p crypto-bigint --precise 0.7.10` | **Rejected.** `elliptic-curve 0.13.8` requires `^0.5` |
| `cargo update -p crypto-bigint --dry-run` | No newer compatible version |
| `[patch.crates-io] crypto-bigint = "=0.7.10"` | **Rejected.** A patch must come from another source, and 0.7 could not satisfy `^0.5` anyway |
| Move to `soroban-sdk` 25.3.0 | Resolves; still `crypto-bigint 0.5.5`, `elliptic-curve 0.13.8` |
| Move to `soroban-sdk` 27.0.6 | Resolves; still 0.5.5 and 0.13.8 |
| Move to `soroban-sdk` 29.0.1 (latest stable) | Resolves; still 0.5.5 and 0.13.8 |
| Move to `soroban-sdk` 30.0.0-rc.1 | Resolves; still 0.5.5 and 0.13.8 |

The SDK rows only tested dependency resolution. They were not compiled, because a move
would be a protocol-level migration with no advisory benefit.

**Where the fix has to come from.** A patched stack does now exist: `elliptic-curve
0.14.3` requires `crypto-bigint ^0.7.8`, and `k256 0.14.0` requires `elliptic-curve
^0.14.1`. However, `soroban-env-host` 30.0.0 and the default branch of
`stellar/rs-soroban-env` still declare `k256 0.13`, `p256 0.13`, `ecdsa 0.16` and
`elliptic-curve 0.13`. So the fix has to land in the Stellar host. This repository
cannot apply it safely, and searches of the Stellar trackers found no issue for this
advisory.

**Conclusion: this cannot currently be eliminated by a compatible upgrade.**

Exposure, as far as it could be checked:

- `cargo tree --workspace --target wasm32v1-none -e normal` has no `crypto-bigint`,
  `elliptic-curve` or `soroban-env-host`, so the vulnerable code is not in contract WASM.
- No Rust source under `contracts/` or `tools/` directly uses `k256`, `p256`,
  `elliptic-curve` or `crypto-bigint`, and `tools/spec-tool/Cargo.lock` has no
  `crypto-bigint`.
- Applicability to 0.5.5 is neither ruled out nor established. `BoxedMontyForm` and
  `ctutils` are absent from its source, and its only `UintRef` is the unrelated
  `der::asn1` type. Subtraction, division and Montgomery operations named by the
  advisory do exist there, and RustSec's range includes the version.
- Validator and wallet software is outside this repository and was not assessed.

The gate stays enabled and `main` stays red on this job. Remaining human choices:

1. Report it upstream to `stellar/rs-soroban-env`, asking for a host migration or a
   statement of applicability.
2. Accept a red job for now, or approve a documented, time-boxed exception with an
   owner and an expiry. This review added no ignore or target filter, and adding one
   is a security-policy decision for a maintainer.
3. Separately decide whether to leave SDK 22, since the SDK's published support policy
   covers its two most recent major releases. That is a compatibility migration, not a
   fix for this advisory.

## 8. Who can resolve what

| Issue | Engineering can do | Needs a person |
|---|---|---|
| Licensing inventory and matrix | **Done** (this file) | Review by counsel |
| Governing license, relicensing, GPL scope | No | Rights holders and counsel |
| Uniswap lineage | Evidence only (section 4) | Counsel |
| Notices and manifest declarations | Keep unchanged; update once decided | Decision on final wording |
| Rewrite of the library | Tests and differential harness | Independent authors and a process set by counsel |
| RustSec upgrade | **Done:** investigated, none available | Upstream report; exception decision |
| Independent math review | Prepare evidence | Independent reviewer; revert decision if it fails |
| CI required-check and bypass policy | No (token cannot read it) | Repository admin |
| Testnet deployment and evidence | Scripts are ready; nothing run | Explicit approval and keys |

## 9. Reproduce

```bash
U=/tmp/soroswap-review; C=6eade008ee6794075ad941609e0fef0d1124d3eb
git clone https://github.com/soroswap/core "$U"

# License declaration on every upstream ref
git -C "$U" for-each-ref --format='%(refname:short)' refs/remotes/origin | grep -v HEAD |
  while read -r r; do git -C "$U" show "$r:contracts/library/Cargo.toml" 2>/dev/null |
    grep -E '^license' | head -1; done | sort | uniq -c

# Per-file history and local delta
p=contracts/library/src/quotes.rs
git -C "$U" log --format='%h %an %ad %s' --date=short "$C" -- "$p"
git diff --no-index --numstat <(git -C "$U" show "$C:$p") "$p"

# Uniswap lineage
git -C "$U" show --stat c012e66 | head -12
gh api repos/Uniswap/v2-periphery --jq .license.spdx_id

# RustSec, in a throwaway copy only
cargo tree --locked -i crypto-bigint --target all
cargo update -p crypto-bigint --precise 0.7.10
sed -i -E 's/^(soroban(-token)?-sdk = ")[^"]+/\1=29.0.1/' Cargo.toml
cargo generate-lockfile && grep -A1 'name = "crypto-bigint"' Cargo.lock
```
