# Contract reference

The authoritative, machine-readable interface is in
[`metadata/*.json`](../metadata) (generated from the WASM) and the generated
TypeScript clients in [`bindings/`](../bindings). This page explains them.

All amounts are `i128` in the token's smallest unit. Errors are returned as
contract errors with the numeric codes below. Error type names and event
topics keep their upstream `Soroswap*` names on purpose, so existing decoders
and indexers keep working.

## Deployment initialization

New factory artifacts require `__constructor(setter: Address, pair_wasm_hash:
BytesN<32>)`; new router artifacts require `__constructor(factory: Address)`.
Soroban invokes these atomically during creation, before another transaction can
call the instance. Missing arguments fail creation. Deployers must use
CreateContractV2/`deploy_v2` or the CLI constructor arguments after `--`.
The compatibility `initialize` methods are already consumed on new deployments.
Existing immutable deployments retain their previous behavior. Pair deployment
is unchanged: the factory creates and initializes it in one invocation.

---

## VELTRIXAE Router

**Integrators should only call the router.**

### State-changing methods (liquidity/swaps require `to` signature)

| Method | Arguments | Returns | Notes |
|---|---|---|---|
| `initialize` | `factory: Address` | `()` | Compatibility entry point; rejects replay after constructor deployment. |
| `add_liquidity` | `token_a, token_b: Address`, `amount_a_desired, amount_b_desired, amount_a_min, amount_b_min: i128`, `to: Address`, `deadline: u64` | `(amount_a, amount_b, liquidity)` | Creates the pair if missing. Pulls tokens from `to`, mints LP to `to`. |
| `remove_liquidity` | `token_a, token_b: Address`, `liquidity, amount_a_min, amount_b_min: i128`, `to: Address`, `deadline: u64` | `(amount_a, amount_b)` | Burns `liquidity` LP from `to`. |
| `swap_exact_tokens_for_tokens` | `amount_in, amount_out_min: i128`, `path: Vec<Address>`, `to: Address`, `deadline: u64` | `Vec<i128>` amounts per hop | Exact input, minimum output. Multi-hop via `path`. |
| `swap_tokens_for_exact_tokens` | `amount_out, amount_in_max: i128`, `path: Vec<Address>`, `to: Address`, `deadline: u64` | `Vec<i128>` amounts per hop | Exact output, maximum input. |

Checks performed, in order: initialized; non-negative amounts; `to` auth;
`deadline > ledger.timestamp`; (swaps) path has >= 2 tokens, no hop repeats a
token, every hop is a factory-registered pair; slippage bound.

### Read-only methods

| Method | Arguments | Returns |
|---|---|---|
| `get_factory` | | `Address` |
| `router_pair_for` | `token_a, token_b` | deterministic pair `Address` (no existence check) |
| `router_quote` | `amount_a, reserve_a, reserve_b` | `amount_a * reserve_b / reserve_a` (rounded down) |
| `router_get_amount_out` | `amount_in, reserve_in, reserve_out` | output after 0.3% fee |
| `router_get_amount_in` | `amount_out, reserve_in, reserve_out` | required input (rounded up) |
| `router_get_amounts_out` | `amount_in, path` | amounts along `path` |
| `router_get_amounts_in` | `amount_out, path` | amounts along `path` |

Exact-output input quotes round required effective input before grossing up for
the pair's integer-rounded fee. Sufficient historical quotes retain their value;
underfunded quotes can increase. Pair fee policy/invariant and this interface
are unchanged. See [the counterexample and proof](review/PR9_REMEDIATION.md).

### Errors (`CombinedRouterError`)

| Code | Name | Meaning |
|---|---|---|
| 501 | RouterNotInitialized | `initialize` not called |
| 502 | RouterNegativeNotAllowed | negative amount argument |
| 503 | RouterDeadlineExpired | `ledger.timestamp >= deadline` |
| 504 | RouterInitializeAlreadyInitialized | second `initialize` |
| 505 | RouterInsufficientAAmount | token A below `amount_a_min` |
| 506 | RouterInsufficientBAmount | token B below `amount_b_min` |
| 507 | RouterInsufficientOutputAmount | output below `amount_out_min` |
| 508 | RouterExcessiveInputAmount | input above `amount_in_max` |
| 509 | RouterPairDoesNotExist | pair missing (remove liquidity, or any swap hop) |
| 510 | LibraryInsufficientAmount | `quote` with amount <= 0 |
| 511 | LibraryInsufficientLiquidity | zero reserve, or `amount_out >= reserve_out` |
| 512 | LibraryInsufficientInputAmount | `amount_in <= 0` |
| 513 | LibraryInsufficientOutputAmount | `amount_out <= 0` |
| 514 | LibraryInvalidPath | path shorter than 2 |
| 515 | LibrarySortIdenticalTokens | identical tokens in a pair or hop |

### Events

| Topics | Data |
|---|---|
| `("SoroswapRouter", "init")` | `{ factory }` |
| `("SoroswapRouter", "add")` | `{ token_a, token_b, pair, amount_a, amount_b, liquidity, to }` |
| `("SoroswapRouter", "remove")` | `{ token_a, token_b, pair, amount_a, amount_b, liquidity, to }` |
| `("SoroswapRouter", "swap")` | `{ path, amounts, to }` |

---

## VELTRIXAE Factory

| Method | Arguments | Returns | Auth |
|---|---|---|---|
| `initialize` | `setter: Address, pair_wasm_hash: BytesN<32>` | `()` | none (one-time) |
| `create_pair` | `token_a, token_b` | pair `Address` | none (permissionless) |
| `set_fee_to` | `to` | `()` | `fee_to_setter` |
| `set_fee_to_setter` | `new_setter` | `()` | `fee_to_setter` |
| `set_fees_enabled` | `is_enabled: bool` | `()` | `fee_to_setter` |
| `get_pair` | `token_a, token_b` | `Address` | read |
| `pair_exists` | `token_a, token_b` | `bool` | read |
| `all_pairs` | `n: u32` | `Address` | read |
| `all_pairs_length` | | `u32` | read |
| `fee_to` / `fee_to_setter` / `fees_enabled` | | `Address` / `Address` / `bool` | read |

`create_pair` fails (and leaves the registry untouched) if either address is
not a deployed token contract: the new pair calls `symbol()` on both tokens
during initialization.

### Errors (`FactoryError`)

| Code | Name |
|---|---|
| 201 | NotInitialized |
| 202 | CreatePairIdenticalTokens |
| 203 | CreatePairAlreadyExists |
| 204 | InitializeAlreadyInitialized |
| 205 | PairDoesNotExist |
| 206 | IndexDoesNotExist |

### Events

| Topics | Data |
|---|---|
| `("SoroswapFactory", "init")` | `{ setter }` |
| `("SoroswapFactory", "new_pair")` | `{ token_0, token_1, pair, new_pairs_length }` |
| `("SoroswapFactory", "fee_to")` | `{ setter, old, new }` |
| `("SoroswapFactory", "setter")` | `{ old, new }` |
| `("SoroswapFactory", "fees")` | `{ fees_enabled }` |

---

## VELTRIXAE Pair

Deployed only by the factory. **Do not call `deposit`, `swap`, `withdraw`
directly** unless you transfer tokens in and call the pair atomically in the
same transaction: these methods have no access control and act on the
difference between balances and reserves, so tokens sent in a separate
transaction can be taken by anyone.

| Method | Arguments | Returns |
|---|---|---|
| `initialize` | `factory, token_0, token_1` (requires `token_0 < token_1`) | `()` |
| `deposit` | `to` | LP minted |
| `swap` | `amount_0_out, amount_1_out: i128, to` | `()` |
| `withdraw` | `to` (LP must already be sent to the pair) | `(amount_0, amount_1)` |
| `skim` | `to` | sends `balance - reserve` of each token to `to` |
| `sync` | | sets reserves to balances |
| `get_reserves` | | `(reserve_0, reserve_1)` |
| `token_0` / `token_1` / `factory` / `k_last` | | |

LP token (SEP-41): `name`, `symbol`, `decimals` (7), `total_supply`,
`balance`, `allowance`, `approve`, `transfer`, `transfer_from`, `burn`,
`burn_from`. Name: `<SYM0>-<SYM1> VELTRIXAE LP Token`, symbol:
`<SYM0>-<SYM1>-VLX-LP` (each symbol truncated to 6 characters).

### Errors (`SoroswapPairError`)

| Code | Name | | Code | Name |
|---|---|---|---|---|
| 101 | InitializeAlreadyInitialized | | 110 | SwapInsufficientLiquidity |
| 102 | NotInitialized | | 111 | SwapInvalidTo |
| 103 | InitializeTokenOrderInvalid | | 112 | SwapInsufficientInputAmount |
| 104 | DepositInsufficientAmountToken0 | | 113 | SwapNegativesInNotSupported |
| 105 | DepositInsufficientAmountToken1 | | 114 | SwapKConstantNotMet |
| 106 | DepositInsufficientFirstLiquidity | | 115 | WithdrawLiquidityNotInitialized |
| 107 | DepositInsufficientLiquidityMinted | | 116 | WithdrawInsufficientSentShares |
| 108 | SwapInsufficientOutputAmount | | 117 | WithdrawInsufficientLiquidityBurned |
| 109 | SwapNegativesOutNotSupported | | 118 | UpdateOverflow |

Arithmetic overflow (e.g. a first deposit where `amount_0 * amount_1 >
i128::MAX`) aborts the invocation with a host error rather than a typed code.

### Events

| Topics | Data |
|---|---|
| `("SoroswapPair", "deposit")` | `{ to, amount_0, amount_1, liquidity, new_reserve_0, new_reserve_1 }` |
| `("SoroswapPair", "swap")` | `{ to, amount_0_in, amount_1_in, amount_0_out, amount_1_out }` |
| `("SoroswapPair", "withdraw")` | `{ to, liquidity, amount_0, amount_1, new_reserve_0, new_reserve_1 }` |
| `("SoroswapPair", "sync")` | `{ new_reserve_0, new_reserve_1 }` |
| `("SoroswapPair", "skim")` | `{ skimmed_0, skimmed_1 }` |

The LP token also emits the standard token `mint`, `burn`, `transfer` and
`approve` events.
