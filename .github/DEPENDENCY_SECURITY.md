# Dependency security controls

Dependabot proposes updates for the ecosystems listed in .github/dependabot.yml. npm and Cargo version updates wait seven days; security updates are not delayed by that cooldown. Updates require review and passing checks; no automatic merge is configured here.

## Checks

- **Dependency review** rejects newly introduced moderate-or-higher advisories in pull requests, across runtime, development and unknown scopes. It does not scan for all malicious code.
- **Installed dependency audit** runs on pull requests, main pushes and daily schedules. It installs from lockfiles with lifecycle scripts disabled and fails for moderate-or-higher npm advisories, including development dependencies. Registry failures also fail the check.
- GitHub Actions are pinned to full commit IDs. Dependabot tracks those references; review upstream diffs before accepting updates. Checkout credentials are not persisted for dependency code to read.
- Existing RustSec checks remain in force where present. Known unresolved advisories are not made safe by this change; consult the repository's dependency-security records.
- **Strict Cargo audit** runs against both lockfiles (the contract workspace and the binding generator) and must pass with no advisory suppressions, ignores or severity-threshold changes.

## crypto-bigint advisory (RUSTSEC-2026-0387)

crypto-bigint 0.5.5 was flagged by the strict Cargo audit job. The affected path is native-only: soroban-sdk 22.0.11 → soroban-env-host 22.1.3 → elliptic-curve 0.13.8 → crypto-bigint 0.5.5 (also reached through ECDSA/k256/p256). `cargo tree --locked --offline -i crypto-bigint` shows this host/test path, while `cargo tree --locked --target wasm32v1-none -i crypto-bigint` shows nothing. The installed elliptic-curve manifest requires crypto-bigint 0.5, so a direct lockfile bump to the patched 0.7.6 is incompatible.

Resolution is to adopt an upstream-reviewed compatible fix or deliberately migrate the coupled SDK/host/curve dependencies together, rather than forcing a lockfile bump. The migration must preserve public interfaces, pass all contract, auth and rounding tests, regenerate bindings and metadata, and leave both lockfiles passing strict Cargo audit. The advisory is not suppressed and is not recorded as a reviewed exception; a passing audit must reflect an actual dependency change.

The binding-generator lockfile failure is tracked separately and repaired on codex/fix-contract-ci. Licensing, the funded Testnet pool and wallet-signed swap evidence remain independent release blockers.

## Repository settings required for enforcement

After merging the workflow, enable the dependency graph, Dependabot alerts and Dependabot security updates in GitHub Settings → Advanced Security. Configure a main-branch ruleset requiring **Dependency review** and **Installed dependency audit** to pass, require pull requests and block force pushes. Confirm any existing required checks and bypass permissions before changing rules. Workflow files alone do not prevent an administrator or an unprotected branch from merging a failing check.

These settings were not enabled or verified by this change. The available GitHub connector does not expose their administration, and local GitHub CLI authentication was unavailable. They must be completed by an authenticated repository administrator.

## Response to an alert

Inspect the advisory, affected lockfile and dependency path. Upgrade to a compatible fixed version and run the repository's tests/build. Preserve upstream notices. Do not add ignores, weaken severity thresholds or use continue-on-error merely to obtain a passing badge. Review any exception separately, including justification, affected scope, owner and expiration.

No dependency scanner guarantees protection against new malware, compromised maintainers, malicious install/build scripts, or harmful application code. Keep credentials out of untrusted workflows and review lockfile, workflow and source changes.
