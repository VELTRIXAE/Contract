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

1. **Legacy initialization race; constructor fix for new artifacts.** Factory
   and router now require constructor arguments and initialize in the deployment
   transaction. `initialize` retains its signature and one-time guard, so later
   takeover attempts fail. Compiled-WASM `deploy_v2` tests cover missing arguments,
   rollback and takeover without mocked authorization. This changes the
   deployment interface, not the AMM formulas. No existing immutable contract is
   retroactively fixed. Legacy unauthenticated initialization remains unsafe;
   never infer safety from a post-initialization read. Pair creation already
   deploys and initializes atomically inside factory.create_pair; standalone
   pair deployments are not the supported user flow.
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
9. **SDK 22.** soroban-sdk 22.0.11 is pinned; its host/testing graph has
   the unresolved advisory documented below. Newer SDK majors exist. Upgrading changes the host interface and must be done deliberately
   with a full test run (Dependabot groups SDK updates for this reason).

## Secrets

- Never commit secret keys, seed phrases or `.env` files (they are
  gitignored).
- Scripts take a **Stellar CLI keystore name** (`stellar keys generate`),
  never a raw secret, and refuse arguments that look like secret keys.
- Scripts refuse mainnet unconditionally.

## Maintainer verification (2026-11-04)

See [baseline review](docs/review/BASELINE_AUDIT.md) and
[Testnet evidence](docs/TESTNET_VERIFICATION.md). This remains an internal
review; no independent audit or verified project deployment is implied.
Public RPC network identity was checked, but no contract IDs/receipts were
available. The constructor fix covers newly deployed artifacts; legacy deployment safety
and the upstream library licensing conflict still require maintainer review.

Transaction scripts now pin the public Testnet RPC URL and passphrase rather
than trusting local CLI aliases. Getter calls explicitly use `--send=no`;
simulation does not persist TTL extensions. Existing deployment records cannot
be overwritten by the deploy script. Remediation handles existing recorded
instances only and requires explicit interactive approval before submissions. Constructor manifests never attempt initialization recovery.
Offline tests verify guards; the remediation flow has not run against live
contracts. Signature nonce replay enforcement belongs to the host and has not
been proved by this repository's mocked authorization tests.

Uploaded pair WASM has its own TTL. Factory activity refreshes the stored hash
value, not necessarily that uploaded code. Before the first pool (or when all
code users are inactive), archived pair code can block new pair creation even
with a live factory. A regression test verifies failure leaves the registry
unchanged. Operators must inspect/restore/extend code with separately approved
transactions; this review does not perform them.

## Open dependency advisory: RUSTSEC-2026-0387

The 2026-11-04 cargo-audit 0.22.2 run fails for `crypto-bigint 0.5.5`:
[upstream advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html).
The advisory describes timing-dependent behavior in intended constant-time
integer operations; practical impact depends on downstream secret arithmetic
and side-channel access. Do not interpret this as demonstrated AMM fund loss.

`cargo tree --locked -i crypto-bigint` traces it through `elliptic-curve 0.13.8`
and `soroban-env-host 22.1.3` in the host/testing SDK graph. The normal
`wasm32v1-none` graph has no such dependency. The advisory requires >=0.7.6,
while elliptic-curve requires 0.5; there is no compatible lockfile-only update.
A reviewed SDK/host migration or upstream compatible backport is required.
No ignored advisory, local cryptography patch or unsupported major override was
added. CI deliberately remains failing until this is resolved. The generator
lockfile passes; npm reports zero vulnerabilities for all three bindings.
`derivative` and `paste` also produce unmaintained dependency warnings.
See [dependency investigation](docs/DEPENDENCY_SECURITY.md) and final
[verification results](docs/review/VERIFICATION_RESULTS.md).

Full advisory investigation: [DEPENDENCY_SECURITY.md](docs/DEPENDENCY_SECURITY.md).
Licensing evidence: [LICENSING_REVIEW.md](docs/LICENSING_REVIEW.md).
