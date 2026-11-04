# Maintainer review baseline

Date: 2026-11-04. Starting commit: `499fef4`. Branch:
`funding-readiness/contracts` (already existed, clean at start).
This is an internal engineering review, not an independent security audit.
This file records the initial baseline. The follow-up constructor fix and final
results in VERIFICATION_RESULTS.md supersede open initialization findings below.

## Inspected scope

The 141 tracked files were inventoried; review covered six contract crates,
all test modules, three generated bindings and lockfiles, interface metadata,
independent spec-tool workspace, scripts, workflows, deployment directory,
licenses, NOTICE and documentation. See `baseline-inventory.json` for paths and
sizes. Available local git history and all refs were inspected; no history was
rewritten. No credentials were read from local keystores or cloud directories.

Baseline `make build`, `make test` (253 tests), and `make lint` exited 0.
Test helper deprecation warnings remain. Final run output is stored under
`evidence/`; the old `docs/TECHNICAL_AUDIT.md` is a dated historical report.

## Findings and disposition

| Finding | Impact | Disposition |
|---|---|---|
| Floating Rust stable, unlocked Make/script builds | Different compiler/dependency resolution can alter artifacts | Pin installed Rust 1.99.1; use locked builds; compare clean rebuild hashes |
| Local CLI network alias trusted | An alias named testnet can point elsewhere | Explicit public Testnet RPC and passphrase in all transaction/verification scripts; reject local/mainnet |
| Deployment command options appended after method `--` | Source/network flags can become method arguments | Insert options before method separator; capture stderr synchronously |
| Getter invocations in deployment/pool scripts not explicitly simulation-only | A getter can extend TTL and thus change state | Explicit `--send=no` for getters |
| Existing deployment manifest could be overwritten | Lost provenance and accidental redeployment | Refuse deployment if manifest already exists |
| No tracked deployment IDs/receipts in available history | Cannot verify project contracts on chain | Record exact blocker; do not infer global absence of deployments |
| First initialization is unauthenticated | First caller can choose factory admin or router factory | Existing documented high-risk limitation; regression tests demonstrate it; constructor redesign needs a separate reviewed change |
| Library upstream GPL-3.0 manifest versus repository Apache-2.0 | Unresolved distribution/release licensing conflict | Preserve notices and flag for upstream/legal clarification; not resolved by vendoring |
| No independent security audit | Mainnet/value use remains unsupported | Preserve explicit unaudited status |
| No advisory or artifact consistency CI gates | Dependency or generated artifact drift can go unnoticed | Add Rust/npm advisory checks, metadata comparison, clean rebuild hash checks |
| Partial deployment logs are temporary and hashes parsed heuristically | A failed deployment may lose IDs or misidentify transaction hashes | Remains open; issue draft specifies durable journal and structured receipts |

## Authority and storage review

Factory `set_fee_to`, `set_fee_to_setter`, and `set_fees_enabled` require the
current setter. Handover changes that signer. Fees start disabled; the setter
can enable and redirect protocol fee growth. There is no pause, rescue or
code-upgrade entry point in the production factory/pair/router. The test token
has a mint/admin authority and must not be represented as a production asset.

Router liquidity and swap methods require `to` authorization. Pair pool methods
are deliberately permissionless and consume balances already transferred to the
pair. Direct transfers outside an atomic router invocation expose excess funds
to other callers. No AMM arithmetic or production ABI was changed by this review.

Instance TTL targets are 30 * 17,280 ledgers (threshold 29 days). Factory registry
and pair code-hash entries target 60 days (threshold 59). LP balances target
120 days (threshold 119). LP allowances are temporary and expire by ledger
sequence. These are ledger counts, not wall-clock guarantees. Instance/code and
persistent data have different lifetimes; activity does not refresh every
registry index or every inactive holder. Archival/restore must be handled by
operators and clients. Simulation-only getters do not persist TTL extensions.
Factory activity renewal has an automated ledger-advance regression test;
network archival/restore remains untested.

Initialization replay is tested at contract level. Signed authorization replay
uses Soroban host nonce/expiration enforcement; mocked authorization tests do
not prove signed-envelope replay resistance. A dedicated real-signature/host
integration test is a follow-up, not an existing guarantee.

## Evidence limits

Heuristic scans of the working tree and all 283 reachable historical blobs found
no recognized Stellar seeds, private-key headers, AWS access IDs or GitHub
credential patterns. Values are never printed. This does not establish absence
of arbitrary credentials, seed phrases, external forks, or unreachable history.
Public network identity was checked; project contract state was not verified.

Uploaded pair WASM has its own TTL. Factory activity refreshes the stored hash
value, not necessarily that uploaded code. Before the first pool (or when all
code users are inactive), archived pair code can block new pair creation even
with a live factory. A regression test verifies failure leaves the registry
unchanged. Operators must inspect/restore/extend code with separately approved
transactions; this review does not perform them.

## Advisory result discovered during this review

Workspace cargo-audit: **FAIL**, RUSTSEC-2026-0387 in crypto-bigint 0.5.5;
SDK host/test dependency chain, absent from normal WASM graph. Fixed versions
>=0.7.6 do not satisfy elliptic-curve 0.13.8's 0.5 requirement. Preserve the
failing security gate pending SDK/host migration or upstream backport. The
separate spec-tool lockfile passes. See SECURITY.md for scope and evidence.
