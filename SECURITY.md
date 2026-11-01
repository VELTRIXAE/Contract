# Security

## Status

**These contracts are not audited.** Soroswap Core, from which they are
derived, was audited by OtterSec (report dated 2024-02-22, in the upstream
repository). That audit covers the upstream code at that time only. It does
**not** cover:

- the migration from soroban-sdk 20 to 22,
- the VELTRIXAE changes listed in [docs/UPSTREAM_ATTRIBUTION.md](docs/UPSTREAM_ATTRIBUTION.md),
- any later upstream change.

The review summarized below was an internal, security-focused code review
during the VELTRIXAE migration. It is not a substitute for an independent
audit. Get one before any mainnet deployment or before holding real value.

## Reporting a vulnerability

Please **do not open a public issue**. Use GitHub's private vulnerability
reporting on this repository (Security tab -> "Report a vulnerability"), with
a description, affected contract and function, and a reproduction (ideally a
failing test). We aim to acknowledge reports within 72 hours.

## Review summary (VELTRIXAE migration)

| Area | Finding | Status |
|---|---|---|
| Quote math | `get_amount_in` with `amount_out >= reserve_out` divided by zero or returned a **negative** input amount (`router_get_amount_in`, `router_get_amounts_in`). Pair-level K check prevented fund loss. | **Fixed**, returns `LibraryInsufficientLiquidity` (511). Boundary and property tests added. |
| Router routes | Swap through a hop with no registered pair trapped with an opaque host error. | **Fixed**: every hop is checked with `factory.pair_exists` before any transfer; returns `RouterPairDoesNotExist` (509). Repeated tokens return 515. |
| Integer overflow | Release profile for pair and router had `overflow-checks` off (upstream). Code uses `checked_*` almost everywhere, but not in every path. | **Fixed**: one workspace release profile with `overflow-checks = true` for all contracts. |
| Router WASM surface | Linking the library crate exported the stand-alone library contract's functions from the router WASM. | **Fixed**: library contract behind the opt-in `contract` feature. |
| Build reproducibility | Contracts imported other contracts' compiled WASM and a committed binary at build time. | **Fixed**: trait-defined clients; single workspace lockfile. |
| Authorization | Router requires `to.require_auth()` on all state-changing calls; factory admin setters require `fee_to_setter`. | Verified, tests added (no-signature calls fail, auth rooted at router). |
| Unauthorized withdrawals | LP burn only via `to`'s signature through the router; pair `withdraw` only burns LP already held by the pair. | Verified, tests added. |
| Reserve invariants | K non-decreasing, fee rounding up, min liquidity locked. | Verified, unit and property tests added. |
| Upgrades | No upgrade entry points; no admin can replace code. | By design (immutable). |

## Known limitations and risks (accepted, documented)

1. **Unauthenticated `initialize` (factory and router).** The first caller
   sets the factory's fee setter and pair WASM hash, or the router's factory.
   On Soroban, deploy and initialize are separate transactions, so someone
   can initialize a freshly deployed contract first. Mitigation:
   `scripts/deploy-testnet.sh` verifies state immediately after each
   `initialize` and aborts before anything is funded; if the check fails,
   discard the contract and redeploy. A future version could move this into
   a constructor (`__constructor`, soroban-sdk >= 22), which changes the
   deployment interface.
2. **Pair methods are permissionless and balance-based.** `deposit`, `swap`,
   `withdraw` and `skim` act on `balance - reserve`. Tokens sent to a pair
   outside an atomic router call can be claimed by anyone. Always use the
   router.
3. **Unsupported token types.** Fee-on-transfer, rebasing, or tokens whose
   `balance` can decrease without a transfer from the pair break the
   reserve accounting (e.g. `skim` panics when balance < reserve). Only use
   standard SEP-41 tokens / Stellar Asset Contracts. Malicious token
   contracts can make *their own* pairs unusable but cannot reach other
   pairs' funds.
4. **Pair creation is permissionless.** Anyone can create a pair for any two
   token contracts, including scam tokens. Frontends must curate token lists.
5. **Same pair twice in one route.** A path like A -> B -> A is allowed and
   quoted with pre-swap reserves; the user gets less than optimal output but
   the pool is protected by the K check.
6. **Storage archival.** Pairs and balances inactive for 30 days are archived
   and must be restored (`RestoreFootprint`) before use. No data is lost.
7. **Protocol-fee control.** `fee_to_setter` can redirect up to 1/6 of LP
   fee growth to `fee_to`. Use a multisig or governed account for this role.
8. **No pause / emergency stop.** By design there is no admin who can freeze
   pools or move user funds.
9. **SDK 22.** soroban-sdk 22 is still patched (22.0.11) but newer majors
   exist. Upgrading changes the host interface and must be done deliberately
   with a full test run (Dependabot groups SDK updates for this reason).

## Secrets

- Never commit secret keys, seed phrases or `.env` files (they are
  gitignored).
- Scripts take a **Stellar CLI keystore name** (`stellar keys generate`),
  never a raw secret, and refuse arguments that look like secret keys.
- Scripts refuse mainnet unconditionally.
