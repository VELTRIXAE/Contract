# Atomic initialization and reproducible maintainer verification

Factory/router deployment previously exposed an unauthenticated initialization
window. New SDK 22 constructors initialize configuration in the creation
transaction, so a subsequent caller cannot choose the fee setter, pair hash or
router factory. Missing constructor arguments roll back creation. Four tests
exercise the actual release WASM via deploy_v2 without mocked authorization.
Existing callable methods retain their signatures; deployment now requires
constructor arguments. Existing immutable deployments are not retroactively fixed.
AMM formulas, pair address derivation, storage keys and event formats are retained.

## Changes

- Factory/router constructors, release-WASM lifecycle tests and adapted native/
  integration fixtures; seven other auth/TTL/rollback/allowance regressions.
- Regenerated factory/router deployment clients and metadata for constructor args.
- Testnet-only scripts pin RPC/passphrase, use simulation-only getters, refuse
  manifest overwrite and pass atomic constructor args. Legacy remediation remains
  approval-gated; constructor manifests refuse post-deployment init recovery.
- Rust 1.99.1 and locked builds; binding generation preserves maintained package
  files and lockfiles. CI checks credentials, offline guards, formatting, clippy,
  build/tests, regenerated artifacts, reproducibility and RustSec/npm advisories.
- Baseline review, per-test outcomes, Testnet procedure, exact upstream comparison,
  licensing/dependency investigations, CONTRIBUTING and five contributor drafts.
- Original MIT/Apache license texts, NOTICE, copyright and Soroswap/PaltaLabs/
  Uniswap lineage are preserved. Raw logs, credentials and build outputs excluded.

## Validation

PASS: build; **264 Rust tests**; clippy; formatting; three TypeScript builds;
regeneration consistency; byte-identical fresh WASM rebuild; npm audits (zero
vulnerabilities); independent generator RustSec audit; **nine offline script
guards**; heuristic working-tree/history credential scans.
Commands, exit codes, captured output and per-test outcomes:
[VERIFICATION_RESULTS.md](VERIFICATION_RESULTS.md).

FAIL: contract-workspace RustSec audit, **RUSTSEC-2026-0387** in crypto-bigint
0.5.5 via SDK host/test dependencies. Patched >=0.7.6 cannot satisfy the locked
elliptic-curve 0.5 requirement; 0.7.10 dry-run resolution also fails. No unsafe
major override, cryptographic patch, ignored advisory or weakened gate was added.
The normal WASM graph excludes this crate. Unmaintained warnings remain for
derivative and paste. See [dependency investigation](../DEPENDENCY_SECURITY.md).

## Blockers and limits

- The RustSec gate intentionally blocks until a reviewed SDK/host migration or
  upstream backport resolves the affected dependency.
- **Project Testnet deployment is NOT VERIFIED.** No authentic IDs or receipts
  are available. Public network identity is verified; it is not project evidence.
  No deployment, initialization, upload, funding, pool or TTL transaction was sent.
- Upstream library GPL-3.0 manifest conflicts with the repository Apache license.
  No legal resolution or relicensing is claimed; upstream/legal clarification is
  still needed. See [licensing review](../LICENSING_REVIEW.md).
- No independent audit. Legacy initialization risk remains on old immutable
  artifacts. Live CLI constructor deployment, XDR provenance binding and network
  restore are untested. No mainnet/value-use readiness claim is made.

Follow-up drafts cover constructor envelope/legacy migration validation, durable
partial deployment receipts, TTL/restore, deprecated test helpers and signed auth
replay. This PR is for maintainer review and must not be merged automatically.
