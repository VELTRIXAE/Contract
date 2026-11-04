# Upstream comparison

Compared 2026-11-04 against Soroswap Core commit
[`6eade008ee6794075ad941609e0fef0d1124d3eb`](https://github.com/soroswap/core/commit/6eade008ee6794075ad941609e0fef0d1124d3eb).
The public repository was cloned read-only and the exact tree extracted.
`review/evidence/upstream-files.json` records SHA-256 hashes of its contract files.
See [UPSTREAM_ATTRIBUTION.md](UPSTREAM_ATTRIBUTION.md) for original authorship.
The constant-product AMM is Soroswap/Uniswap V2 derived, not original VELTRIXAE math.

## Reproduce the comparison

```bash
git clone --no-checkout https://github.com/soroswap/core.git /tmp/soroswap-review
git -C /tmp/soroswap-review show --no-patch 6eade008ee6794075ad941609e0fef0d1124d3eb
mkdir -p /tmp/soroswap-tree
git -C /tmp/soroswap-review archive 6eade008ee6794075ad941609e0fef0d1124d3eb contracts LICENSE | tar -x -C /tmp/soroswap-tree
diff -ru /tmp/soroswap-tree/contracts contracts
# diff exits 1 for expected differences; inspect rather than suppress them.
git log --reverse --oneline -- contracts Cargo.toml
```

## Unchanged design and intentional compatibility

Reserve accounting, liquidity mint/burn formulas, 0.3% swap fee and protocol-fee
calculation, minimum liquidity lock, deterministic pair salt/address derivation,
storage keys, existing public method signatures and upstream event prefixes are retained.
Rust `Soroswap*` type names remain for client/spec compatibility. Permissionless
pair creation and pool methods, no pause,
and no code upgrade entry point are inherited behaviors, including their risks.
Error values retained compatibility; router path validation now actively returns
existing typed errors. Identical tokens and invalid routes remain rejected.

## VELTRIXAE changes and provenance

| Change | History / source | Security implication |
|---|---|---|
| Reject output >= reserve in get_amount_in | `4941ffd`, library quotes | Avoid divide-by-zero/negative input quotes |
| Validate every swap hop via factory | `13a065e`, router validate_path | Fail before transferring funds for missing/identical-token hops |
| VELTRIXAE LP name/symbol and metadata | `466e0c1`, `53d02d5` | Branding only; not ownership of upstream math |
| Single workspace/lockfile/release profile | `c97b87c`, root Cargo.toml | Pair/router gain overflow checks; retain checked arithmetic |
| SDK token client and trait clients | `2814db0`, `7d30368`, `c10222c` | Remove build-time imports of committed/dependent WASM; ABI maintained |
| Library export feature gate | `4e4ced3` | Avoid unintended library entry points in router WASM |
| New boundary/property/auth/path tests | `2b5c64c`, `c8b09a1`, `2f753e8`, `d3fcee8` | Extend upstream regression coverage |
| Rust formatting and warning cleanup | `5e3e2b8` and attribution list | Source normalization, no AMM redesign |
| Exact-output rounding correction | PR #9 follow-up, library quotes | Covers integer fee rounding; retains sufficient historical input quotes; pair invariant/fee policy unchanged; independent review pending |
| Current maintainer review | This branch | Locked build/toolchain, script guards, evidence, rollback/TTL tests and CI; atomic factory/router constructors; no AMM formula changes |

## SDK migration

Upstream manifests mix SDK 20.2 and 22 release-candidate declarations. This tree
uses soroban-sdk/token-sdk 22.0.11 resolved in the workspace lockfile and targets
`wasm32v1-none`. Factory pair creation uses `deploy_v2(..., ())` and retains pair address
derivation. Factory/router deployment now adds constructor arguments, removing
the separate initialize transaction for new artifacts. Existing callable methods
retain their signatures; constructors extend the deployment spec. SDK 21+
event-test semantics require inspecting events before the next invocation.
Test-only methods are gated from release WASM. New SDK majors require a deliberate
migration, host compatibility review and regenerated specs/bindings; protocol 30
Testnet identity alone does not prove these artifacts have executed successfully.

## Licensing and audit boundaries

Retain Apache-2.0, NOTICE and original authorship for imported code, and MIT where
applicable to VELTRIXAE additions. The independent spec-tool manifest currently
declares Apache-2.0. Upstream library Cargo.toml explicitly declares GPL-3.0,
contradicting the repository-level Apache license. Vendoring does not resolve
that conflict; obtain upstream clarification/legal review before asserting
unambiguous distribution rights. This review does not relicense upstream code.
The upstream OtterSec audit does not cover VELTRIXAE or its SDK migration.

See [LICENSING_REVIEW.md](LICENSING_REVIEW.md) for preserved texts and exact
manifest evidence; no library relicensing claim is made.
