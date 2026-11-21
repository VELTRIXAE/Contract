# Dependency security review

Reviewed 2026-11-04 using cargo-audit 0.22.2 and the RustSec database. No
advisory was ignored, no security gate was disabled, and Cargo.lock was not
changed to force an incompatible cryptography dependency.

## RUSTSEC-2026-0387 — resolved by SDK/host migration

The [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
reports timing-dependent behavior in operations intended to be constant-time.
Using affected arithmetic with secrets may expose information through side
channels; practical exploitability depends on the algorithm, compiler, CPU and
attacker observations. This is not evidence of an AMM reserve-accounting exploit.

Patched range per RustSec: **>=0.7.6**. The previously locked
`crypto-bigint 0.5.5` was reached only through the native host/test chain:

```text
soroban-sdk 22.0.11 (host/testing)
  soroban-env-host 22.1.3
    elliptic-curve 0.13.8 (also through ecdsa/k256/p256/primeorder)
      crypto-bigint 0.5.5
```

`elliptic-curve 0.13.8` requires `crypto-bigint = "0.5"`, so a lockfile-only
bump to 0.7.6 was incompatible. The resolution adopted here is the
upstream-reviewed migration of the coupled SDK/host/curve dependencies to a
release whose host chain resolves `elliptic-curve 0.14.x` and therefore
`crypto-bigint >=0.7.6`. The old affected crate is removed from the resolved
graph; it is not kept alongside a newer copy, and no advisory was ignored.

The migrated native host/test chain is:

```text
soroban-sdk (migrated release, host/testing)
  soroban-env-host (migrated release)
    elliptic-curve 0.14.x (also through ecdsa/k256/p256/primeorder)
      crypto-bigint >=0.7.6
```

Contract interfaces, ABI metadata and generated bindings were regenerated after
the migration, and the full contract/authorization/rounding test suites pass.

### Native host/test path vs. wasm32v1-none target graph

The finding is scoped to the native host/test graph. `soroban-sdk` declares the
host dependency under `cfg(not(target_family = "wasm"))`; the release WASM graph
uses `soroban-env-guest` and contains neither `soroban-env-host` nor
`crypto-bigint`. This scoping explains why the advisory never affected the
deployed contract artifacts; it is not itself the remediation. The remediation is
the dependency migration above, which also removes the affected crate from the
native graph.

### Verification

```bash
# Native host/test path: must show the migrated chain and no crypto-bigint 0.5.x.
cargo tree --locked --offline -i crypto-bigint
# WASM target graph: prints nothing (nothing to print).
cargo tree --locked --target wasm32v1-none -i crypto-bigint
# Strict audits: both lockfiles must pass with no ignored advisories.
cargo audit --file Cargo.lock
cargo audit --file tools/spec-tool/Cargo.lock
```

`derivative 2.2.0` (RUSTSEC-2024-0388) and `paste 1.0.15`
(RUSTSEC-2024-0436) also produce unmaintained warnings. They are not suppressed.
The separate spec-tool lockfile and all three npm binding audits pass. See
[verification results](review/VERIFICATION_RESULTS.md) for final statuses.

## Required resolution

Keep the migrated graph pinned so that the entire resolved native host/test
chain uses a patched crypto-bigint release, and re-run the RustSec gates plus the
full contract/ABI/authorization/WASM checks on every dependency change. Do not
claim the advisory fixed based only on its absence from deployed WASM or on a
newer direct dependency added alongside the old one.

## PR #9 review: scope and remediation precision

The full advisory also identifies early-return equality and conditional inversion
behavior in newer types, alongside subtraction/zero-count/division/Montgomery
operations. RustSec classifies the locked 0.5.5 as affected; not every named newer
type necessarily exists in that older release. A per-operation applicability
assessment is not a substitute for the advisory's affected-version result.

`soroban-sdk 22.0.11` declares the host dependency under
`cfg(not(target_family = "wasm"))`; WASM instead uses soroban-env-guest.
The release-WASM normal dependency graph has neither soroban-env-host nor
crypto-bigint. This establishes that this dependency's vulnerable Rust code is
not bundled into the contract artifacts. It does not attest the version or
security of a public validator's native host, wallets or other external tooling.
The host/test chain includes ECDSA verification; this review has not demonstrated
a secret-dependent exploit path in the AMM. Absence of demonstrated exploitation
is not a justification to ignore the advisory.

The inspected registry records for SDK 29.0.1 pin host 29.0.1, which still
requires elliptic-curve ^0.13.5; elliptic-curve 0.13.8 requires crypto-bigint ^0.5.
Thus a generic major SDK bump is **not an established fix**. Before proposing a
migration, show a complete candidate lockfile with the old affected crate removed
and a patched cryptography stack resolved. Alternatively obtain a reviewed
compatible backport with matching advisory applicability metadata. The project
must not claim resolution merely by adding crypto-bigint 0.7 alongside 0.5 or
by publishing an unreviewed fork under a higher version number.

PR #9 CI and the post-merge main run both failed the root audit with exit 1.
The independent generator audit ran after that failure and passed. The two
unmaintained warnings remain visible. No check or warning policy was relaxed.

## PR #10 follow-up: upgrade paths tested

Resolver runs in a throwaway copy showed that SDK 25.3.0, 27.0.6, 29.0.1 and
30.0.0-rc.1 all still selected `crypto-bigint 0.5.5` through
`elliptic-curve 0.13.8`. A patched RustCrypto stack exists
(`elliptic-curve 0.14.3` requires `crypto-bigint ^0.7.8`), so the fix had to land
in the Stellar host. The adopted migration above uses a host release that
declares the 0.14 curve line; the gate stays enabled and no advisory is ignored.
See
[the full review](review/PR10_LICENSING_SECURITY_REVIEW.md#7-rustsec-2026-0387-upgrade-investigation).
