# Dependency security review

Reviewed 2026-11-04 using cargo-audit 0.22.2 and the RustSec database. No
advisory was ignored, no security gate was disabled, and Cargo.lock was not
changed to force an incompatible cryptography dependency.

## RUSTSEC-2026-0387 — unresolved

The [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
reports timing-dependent behavior in operations intended to be constant-time.
Using affected arithmetic with secrets may expose information through side
channels; practical exploitability depends on the algorithm, compiler, CPU and
attacker observations. This is not evidence of an AMM reserve-accounting exploit.

The locked affected package is `crypto-bigint 0.5.5`. The dependency chain is:

```text
soroban-sdk 22.0.11 (host/testing)
  soroban-env-host 22.1.3
    elliptic-curve 0.13.8 (also through ecdsa/k256/p256/primeorder)
      crypto-bigint 0.5.5
```

Patched range per RustSec: **>=0.7.6**. The inspected crates.io index cache
contains non-yanked 0.7.9 and 0.7.10; earlier patched 0.7.6–0.7.8 are marked
yanked in that snapshot. The cached 22.x SDK series ends at 22.0.11. These are
registry observations, not a recommendation to force a major dependency update.
The public crates.io metadata API returned HTTP 403; the cargo registry cache,
locked dependency manifests, RustSec and resolver output were inspected instead.

`elliptic-curve 0.13.8` requires `crypto-bigint = "0.5"`; SDK 22.0.11 pins
soroban-env-host to 22.1.3. The patched range cannot satisfy that requirement.
A non-mutating resolver experiment confirms the incompatibility:

```bash
cargo update --offline --dry-run -p crypto-bigint --precise 0.7.10
# FAIL (101): requirement crypto-bigint = "^0.5";
# candidate 0.7.10 does not match; required by elliptic-curve 0.13.8.
```

There is no safe lockfile-only fix within this dependency graph. Resolution
requires a reviewed SDK/host/elliptic-curve migration or an upstream compatible
backport. Vendoring an unreviewed cryptographic patch or overriding the major
version is not a safe remediation. The advisory remains a CI-blocking FAIL.

The normal release WASM dependency graph excludes crypto-bigint. The vulnerable
chain appears in host/test tooling, whose risk must still be assessed:

```bash
cargo tree --locked -i crypto-bigint
cargo tree --locked --target wasm32v1-none -e normal -i crypto-bigint
# Second command: warning: nothing to print.
cargo audit --file Cargo.lock
cargo audit --file tools/spec-tool/Cargo.lock
```

`derivative 2.2.0` (RUSTSEC-2024-0388) and `paste 1.0.15`
(RUSTSEC-2024-0436) also produce unmaintained warnings. They are not suppressed.
The separate spec-tool lockfile and all three npm binding audits pass. See
[verification results](review/VERIFICATION_RESULTS.md) for final statuses.

## Required resolution

Prepare a dedicated SDK/host migration, verify that its entire resolved graph
uses a patched crypto-bigint release, then run the same RustSec gates and full
contract/ABI/authorization/WASM checks. Obtain an upstream backport if keeping
SDK 22 is required. Do not claim the advisory fixed based only on its absence
from deployed WASM or on a newer direct dependency added alongside the old one.

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

Resolver runs in a throwaway copy show that SDK 25.3.0, 27.0.6, 29.0.1 and
30.0.0-rc.1 all still select `crypto-bigint 0.5.5` through `elliptic-curve 0.13.8`.
A patched RustCrypto stack now exists (`elliptic-curve 0.14.3` requires
`crypto-bigint ^0.7.8`), but `soroban-env-host` 30.0.0 and the default branch of
`stellar/rs-soroban-env` still declare the 0.13 line. The fix therefore has to land
in the Stellar host. The advisory stays open and the gate stays enabled. See
[the full review](review/PR10_LICENSING_SECURITY_REVIEW.md#7-rustsec-2026-0387-upgrade-investigation).
