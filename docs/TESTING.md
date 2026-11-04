# Testing

```bash
make test        # build WASM (wasm32v1-none, release) then cargo test
make lint        # clippy correctness lints
```

Integration tests register the **compiled WASM** from
`target/wasm32v1-none/release/`. `cargo test` alone after a contract change
tests stale WASM; `make test` always rebuilds first. CI does the same.

## Suites

Counts are from the last local run (`make test`, Rust 1.99, soroban-sdk
22.0.11): **253 tests, all passing** at the historical baseline. See
[review evidence](review/VERIFICATION_RESULTS.md) for the current run and added regressions.

| Crate | Tests | What they cover |
|---|---:|---|
| `veltrixae-factory` | 60 | initialization, fee-setter auth (mocked and real auth), pair creation, deterministic addresses, events, **security**: no-signature admin calls fail, setter handover, initialize replay, token validation (account address, undeployed contract, non-token contract), registry indexing |
| `veltrixae-pair` | 63 | initialize, deposit, swap, withdraw, skim, sync, protocol fee, LP token, events, **boundaries**: min liquidity edge cases, geometric mean, overflow revert, rounding on add/withdraw, fee ceiling for dust, exact output boundary, reserve drain, invalid `to`, K non-decreasing over a swap sequence |
| `veltrixae-router` | 92 | add/remove liquidity, exact-in/exact-out swaps, multi-hop, library helpers, events, budget, **path validation** (missing pair, missing second hop, repeated token), **security**: missing signatures, auth rooted at router, deadline boundary, slippage moves no funds, 3-hop quotes and zero router custody, initialize replay |
| `veltrixae-library` (unit) | 23 | sort/pair_for, reserves, quote, amount in/out incl. `amount_out >= reserve_out` |
| `veltrixae-library` (property) | 8 | proptest, 2000 cases each: output < reserve, K preserved, quote tightness, amount-in round trip, draining outputs rejected, quote rounds down, monotonicity, typed errors for non-positive input |
| `veltrixae-test-token` | 7 | test token behaviour |

Run one crate or one test:

```bash
cargo test -p veltrixae-router
cargo test -p veltrixae-pair boundaries
cargo test -p veltrixae-library --test amm_properties
cargo test -p veltrixae-router budget -- --nocapture   # CPU/memory budget report
```

Property tests can be made more aggressive locally:

```bash
PROPTEST_CASES=100000 cargo test -p veltrixae-library --test amm_properties
```

## Writing tests

- Prefer `try_*` client calls and assert the exact error code.
- For authorization, call `env.set_auths(&[])` to drop `mock_all_auths()`
  and assert the call fails, or inspect `env.auths()` after a success.
- `env.events().all()` only holds events from the **last** invocation
  (soroban-sdk >= 21); read events before making another call.
- Derive expected values from the formulas and write the arithmetic in a
  comment next to the assertion.
- New AMM math must keep the property tests green; add a property when you
  add a formula.

## Not covered by automated tests

- Real network behaviour (fees, resource limits, archival/restore) is not
  exercised; run `scripts/verify-deployment.sh` after a testnet deploy.
- Fee-on-transfer and rebasing tokens are unsupported and untested (see
  SECURITY.md).

## Maintainer review additions

`contracts/router/src/test/security.rs` now checks rollback after nested LP burn
and underlying-token transfers when withdrawal slippage fails, and rollback of
pair deployment/registry/transfers after an initial deposit fails. Factory/router
first-caller initialization tests explicitly demonstrate the documented risk;
they describe legacy/test-reset state; production constructor deployment is
covered separately below. Factory TTL renewal
is tested after advancing ledger sequence; archived uploaded-code failure leaves
the registry unchanged. LP allowance validity is checked at its expiration ledger
and rejection one ledger later preserves balances, reserves and supply. Existing overflow, invalid pair,
authorization, deadline and reserve-invariant tests remain intact.

Run offline transaction-script guards with:

```bash
python3 -m unittest discover -s scripts/tests -v
```

The fake CLI never contacts a network or signs; passing guards do not establish
live Stellar CLI compatibility. Signed nonce replay and network restore coverage
remain scoped contributor follow-ups.

## Constructor security regressions

Factory/router constructor tests deploy actual compiled release WASM through a
native deployment harness's `deploy_v2`, with no mocked authorization. They
verify correct initial configuration, rejected takeover via initialize, and
rollback of missing-argument creation followed by successful deployment with
the same salt. Four tests supplement the 260-test earlier review baseline.

Inherited native pre-initialization tests deliberately clear constructor-created
storage in test-only fixtures to preserve their assertions about old invalid
states. Those fixtures do not represent a reachable new on-chain deployment.
Cross-contract integration fixtures now pass constructor arguments instead of
performing a separate initialize call. Pair deployment and all math are unchanged.

## PR #9 follow-up

The [remediation report](review/PR9_REMEDIATION.md) records 269 passing Rust tests
and 12 offline script tests. Two additional constructor regressions cover legacy
creation; admin denial is checked on actual constructor-created WASM. A persisted
property counterexample and deterministic/release-router-WASM tests cover
exact-output integer fee rounding. The original round-trip assertion is retained.

`PROPTEST_CASES=100000` is now honored, with a minimum of 2,000 accepted cases
and a proportional budget for rejecting generated inputs outside the bounded
arithmetic domain. CI runs the validated stress command separately; this does
not lower accepted cases, hide a failing assertion or relax a security gate.
