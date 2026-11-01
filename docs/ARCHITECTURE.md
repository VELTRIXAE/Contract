# Architecture

VELTRIXAE is a Uniswap-V2-style constant-product AMM on Soroban, derived from
Soroswap Core. Three deployed contracts and two Rust libraries:

```
            user / frontend / aggregator
                        |
                        v
               +-----------------+      pair_exists / create_pair
               | VELTRIXAE Router| -----------------------------+
               +-----------------+                              |
                 |  token.transfer(user -> pair)                v
                 |  pair.deposit / swap / withdraw     +-------------------+
                 v                                     | VELTRIXAE Factory |
          +----------------+   deploy_v2 + initialize  +-------------------+
          | VELTRIXAE Pair | <-------------------------  fee_to, fees_enabled
          |  (+ LP token)  | ------------------------->  (read by pair)
          +----------------+
                 |
                 v
          SEP-41 token contracts (Stellar Asset Contracts or custom tokens)
```

## Contracts

### Factory (`contracts/factory`)
- `initialize(setter, pair_wasm_hash)` once. Stores the fee setter, sets
  `fee_to = setter`, stores the pair WASM hash.
- `create_pair(token_a, token_b)` sorts the tokens, deploys a pair with
  `deployer().with_current_contract(salt).deploy_v2(pair_wasm_hash, ())`
  where `salt = sha256(xdr(token_0) || xdr(token_1))`, calls
  `pair.initialize(factory, token_0, token_1)`, and records the pair both by
  token pair and by index. Permissionless.
- The fee setter can change `fee_to`, `fee_to_setter` and `fees_enabled`.

### Pair (`contracts/pair`)
- Holds `reserve_0`, `reserve_1`, `k_last`, the token addresses and the
  factory address in instance storage.
- Is also its own LP token (SEP-41 interface: `transfer`, `approve`,
  `balance`, ... in `soroswap_pair_token/`).
- **Pull-free design**: `deposit`, `swap` and `withdraw` do not take tokens
  from anyone. They compare current token balances with stored reserves and
  act on the difference. The caller (the router) must transfer tokens into
  the pair earlier in the same transaction.
- Swap fee: 0.3%, charged on the input with ceiling rounding
  (`fee = ceil(amount_in * 3 / 1000)`), enforced by the K check
  `(balance_0 - fee_0) * (balance_1 - fee_1) >= reserve_0 * reserve_1`.
- First deposit mints `sqrt(amount_0 * amount_1) - 1000` LP tokens and
  locks `MINIMUM_LIQUIDITY = 1000` in the pair itself forever.
- Protocol fee (optional): when `fees_enabled`, 1/6 of LP fee growth is
  minted to `fee_to` on the next deposit or withdraw (Uniswap V2 `mintFee`).

### Router (`contracts/router`)
- The only contract users should call. Requires `to.require_auth()` on every
  state-changing function; `to` both pays and receives.
- Enforces deadlines (`ledger.timestamp < deadline`), slippage bounds
  (`amount_*_min`, `amount_out_min`, `amount_in_max`) and non-negative
  amounts.
- **VELTRIXAE addition**: validates every swap hop against the factory
  (`pair_exists`) before moving any tokens.
- Computes pair addresses locally with `veltrixae-library::pair_for`
  (deterministic deployment address); no factory call needed for lookups.
- Never holds tokens: transfers go user -> first pair -> next pair ... -> user.

## Libraries

- `veltrixae-library`: `sort_tokens`, `pair_for`, `get_reserves_with_factory`,
  `quote`, `get_amount_out`, `get_amount_in`, `get_amounts_out`,
  `get_amounts_in`. The stand-alone library *contract* is behind the
  `contract` feature so linking the library does not export its functions
  from the router.
- `veltrixae-factory-interface`: `SoroswapFactoryTrait`, the generated
  `SoroswapFactoryClient` and `FactoryError`, shared by pair and router.

## Storage and TTL

| Contract | Data                                  | Storage    | TTL extension                              |
|----------|---------------------------------------|------------|--------------------------------------------|
| Factory  | setter, fee_to, fees_enabled, count   | instance   | 30 days on every call                      |
| Factory  | pair WASM hash, pair-by-tokens, pair-by-index | persistent | 60 days when written or read      |
| Pair     | tokens, reserves, factory, k_last, LP metadata | instance | 30 days on every call               |
| Pair     | LP balances                           | persistent | 30 days when read or written               |
| Pair     | LP allowances                         | temporary  | until `expiration_ledger`                  |
| Router   | factory address                       | instance   | 30 days on every call                      |

Instance and persistent entries that expire are archived, not deleted, and
can be restored with a `RestoreFootprint` operation (most clients do this
automatically during simulation). Temporary allowances are gone after expiry.

## Upgradeability

None of the contracts has an upgrade entry point or admin able to replace its
code. Deployed contracts are immutable; fixing a bug means deploying new
contracts and migrating liquidity. The only privileged role is the factory's
`fee_to_setter`, which controls protocol-fee settings only.

## Build pipeline

One cargo workspace, `soroban-sdk 22.0.11`, target `wasm32v1-none`, one
release profile with `overflow-checks = true`. No contract imports another
contract's compiled WASM at build time (clients are defined from traits), so
`cargo build --target wasm32v1-none --release` builds everything in one pass.
Test suites do import the compiled WASM, which is why `make test` builds
first.
