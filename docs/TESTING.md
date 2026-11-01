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
22.0.11): **253 tests, all passing**.

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
