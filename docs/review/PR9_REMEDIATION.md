# PR #9 review and remediation report

Date: 2026-11-04. Reviewed PR head `e4351d79925faf4484a34006131b504a9756e1c8`.
GitHub reports PR #9 merged at 14:09:28 UTC into
`4bb3d0932fe03ac3c0fed07cd6c5b7931814d182`. This review did not merge, revert,
rewrite history, deploy or initialize a public-network contract. Validated fixes
are prepared separately on `review/pr9-remediation` because a merged PR's diff
cannot be extended by pushing new commits to its old branch.

This is an internal engineering review, not an independent audit or legal opinion.
No unresolved dependency or licensing issue is represented as fixed.

## Findings and disposition

| Finding | Impact | Disposition |
|---|---|---|
| RUSTSEC-2026-0387, crypto-bigint 0.5.5 | Conditional timing side channels in host/test cryptography | BLOCKED: no compatible patched dependency in the current graph; no ignore/filter added |
| Conflicting library GPL/Apache declarations | Unclear distribution rights/obligations for imported library and linked router | LEGAL/RIGHTSHOLDER REVIEW: exact source/notices inventoried; no relicensing performed |
| Factory/router constructor compatibility | Existing callers must supply constructor args at deployment | VERIFIED: callable specs unchanged; required creation args documented; V1/V2 creation tests pass |
| Underfunded exact-output quote, found on full rerun | Rare swap failure at pair invariant check; not caused by constructors | DRAFT FIX: round effective input before fee gross-up; independent math review required |
| Verifier ignores failed pair-count getter | Could report success despite incomplete state read | FIXED LOCALLY: unavailable/malformed counts fail verification; offline regressions pass |
| PR merged despite failed advisory job | Security failure did not prevent the observed merge | MAINTAINER REVIEW: inspect required-check/bypass policy; no branch protection changed |
| Missing deployment records | Cannot attest VELTRIXAE Testnet IDs/code/state/receipts | NOT VERIFIED; no evidence invented and no transactions submitted |

## 1. RustSec advisory: exact scope and options

The [full RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html)
and [upstream 0.7.6 release PR](https://github.com/RustCrypto/crypto-bigint/pull/1365)
were inspected. The issue covers timing-dependent branching in intended
constant-time arithmetic, including subtraction, zero counting, division and
Montgomery multiplication, plus equality/inversion behavior in newer types.
Exploitation depends on secret data, algorithm, compiler/CPU and observable
side channels. RustSec classifies this locked version as affected; this review
has not established that every named newer type exists in 0.5.5 or demonstrated
an exploitable AMM secret flow.

```text
soroban-sdk 22.0.11
  soroban-env-host 22.1.3 (native target; also testutils)
    elliptic-curve 0.13.8, ecdsa 0.16.9, k256 0.13.4, p256 0.13.2
      crypto-bigint 0.5.5
```

The SDK's WASM target uses soroban-env-guest instead of the native host.
`cargo tree --locked --target wasm32v1-none -e normal -i crypto-bigint`
prints `warning: nothing to print`. The normal WASM graph also excludes
soroban-env-host. Therefore this dependency's vulnerable Rust implementation is
not bundled in production contract WASM. Public validators' native host code
and external wallets are outside this artifact-level determination.

Patched range: **>=0.7.6**. The inspected registry snapshot marks 0.7.6–0.7.8
yanked, with non-yanked 0.7.9 and 0.7.10. However elliptic-curve 0.13.8 requires
crypto-bigint ^0.5 and SDK 22.0.11 pins host =22.1.3. The dry-run attempt to use
0.7.10 fails resolution without modifying Cargo.lock. A compatible lockfile-only
fix is not available in this graph.

The inspected SDK 29.0.1 record pins host 29.0.1, which still requires
elliptic-curve ^0.13.5. A generic major SDK bump is thus not a verified cure.
A migration is a candidate only after resolving a complete patched stack and
proving removal of 0.5.5. A compatible upstream backport is another candidate,
requiring independent cryptographic review and accurate advisory applicability
metadata. Adding a new direct dependency alongside the vulnerable transitive
copy, blindly overriding semver, or editing ignored-advisory lists is not a fix.

Warnings remain: derivative 2.2.0 / RUSTSEC-2024-0388 and paste 1.0.15 /
RUSTSEC-2024-0436 (unmaintained). No warning policy was relaxed. Further evidence
and reproduction commands: [DEPENDENCY_SECURITY.md](../DEPENDENCY_SECURITY.md).

## 2. Licensing: exact implicated material

Upstream commit: `6eade008ee6794075ad941609e0fef0d1124d3eb`.
Root LICENSE is Apache-2.0, but upstream library Cargo.toml line 9 and README
line 51 state GPL-3.0. The local `contracts/library/README.md` preserves that
GPL notice while the local manifest inherits Apache-2.0. This is conflicting
licensing evidence, not a finding that GPLv3 and Apache-2.0 can never coexist.

The twelve imported local files within that library declaration are:

```text
contracts/library/Cargo.toml
contracts/library/README.md
contracts/library/src/error.rs
contracts/library/src/lib.rs
contracts/library/src/math.rs
contracts/library/src/quotes.rs
contracts/library/src/reserves.rs
contracts/library/src/tokens.rs
contracts/library/src/test.rs
contracts/library/src/test/get.rs
contracts/library/src/test/quote.rs
contracts/library/src/test/tokens.rs
```

The six non-test Rust files are the production implementation scope to review.
`tests/amm_properties.rs` is later VELTRIXAE test work, not an imported file.
The absence of per-file license headers in upstream Rust does not establish a
permissive grant. No existing notice, copyright or author field was removed.

Upstream router depends on published soroswap-library 0.3.0, distinct from the
in-repository upstream library 2.0.0. The published archive was inspected without
executing it: normalized/original manifests both declare GPL-3.0 and README line
51 repeats it; no separate license/notice file is packaged. Archive SHA-256
`26352158999f8f2bbbc70481c0218d6e9cd8cc39f1d3eaa54a133e6b914f4dec`
matches the checksum in the exact upstream router lockfile.

Current Cargo.lock no longer uses that published crate. Instead router links
`veltrixae-library` as a path rlib, incorporating its implementation into router
WASM. Factory/pair have no such dependency edge. Removing the registry package
or renaming its code does not settle source rights. Generated specs/clients need
provenance-specific review; automatic license propagation is not asserted here.

Maintainer/legal options are: obtain a precise rightsholder grant/clarification;
assess a compliant GPLv3 distribution if applicable (scope, license text,
corresponding source/build material and retained notices); replace uncleared
implementation using independently established permissive provenance and review;
or defer affected distribution/release claims. None was selected or implemented.
Original MIT/Apache texts, NOTICE and the local GPL statement remain intact.
[LICENSING_REVIEW.md](../LICENSING_REVIEW.md) records the notices, exact dependency
edges and authoritative compatibility reference. Legal review is needed for
open-source distribution as well as any proposed commercial release.

## 3. Initialization security and compatibility

Production constructor bodies delegate to the original one-time initialize
logic. Factory's setter/hash and router's factory are written before deployment
returns. They do not add a fee-setter signature requirement to creation; the
authorized deployer intentionally chooses the initial configuration. Publishing
an unrelated malicious factory remains possible and is not trusted-ID takeover.

The release-WASM tests use a deployment harness with no mocked authorization:
correct creation, rejected later initialize, missing-argument rollback/retry,
and now deprecated CreateContract/V1 rejection/retry as well. The host source
calls the constructor for both creation paths, so V1 cannot bypass the fix.
Post-constructor unauthorized fee recipient/setter/enable calls also fail while
preserving configuration. Pair creation already deploys/initializes atomically
inside the factory and was not changed by this review.

Metadata compared against pre-PR baseline 499fef4: all existing 21 factory,
33 pair and 20 router spec entries (functions/types/errors) are identical.
Only factory/router __constructor entries were added. Fee policy, LP accounting,
pair derivation, storage keys and event encoding remain unchanged. Deployment
compatibility is deliberately different: SDK/CLI/client creation callers must
supply the new args; callable initialize now rejects on a constructor-created
instance. Existing immutable deployments retain their original behavior and
cannot be claimed fixed by this source change.

Native pre-initialization tests deliberately reset test storage and do not
represent a reachable production lifecycle. Actual release-WASM tests separately
cover the lifecycle boundary. Live CLI/network creation remains untested.

## 4. New exact-output counterexample and draft correction

The first full rerun failed the existing amount_in_round_trips property with
this shrunk input. quotes.rs was unchanged between baseline 499fef4 and PR #9,
so the failure was pre-existing, not introduced by constructors:

```text
reserve_in   = 72320497631014911
reserve_out  = 94716774993543284
desired_out  = 28883596383956043
old input    = 31825313508430334
old max out  = 28883596383956042  (one unit short)
fixed input  = 31825313508430335
fixed max out= 28883596383956043
```

Pair fee is ceil(3g/1000), so effective input for integer gross g is
floor(997g/1000). For desired output o and positive reserves Rin, Rout, the
invariant requires effective input >= ceil(Rin*o/(Rout-o)). Hence a sufficient
integer gross input is:

```text
required_net   = ceil(Rin * o / (Rout - o))
required_gross = ceil(1000 * required_net / 997)
new_quote      = max(historical_quote, required_gross)
```

The historical quote and sufficient existing quotes retain their exact values.
Only an underfunded quote increases. All operations remain checked. The new
required_net is <= Rin*o for positive integer Rout-o, so its multiplication by
1000 cannot introduce overflow if the old checked numerator already fits.
Input/liquidity error guards and codes remain unchanged. The pair invariant,
fee rate and LP formulas are untouched; the quote inverse is explicitly changed
and attributed, not described as unchanged AMM math.

The old behavior can abort an exact-output swap at the pair K check. The pair
and router atomicity protect reserves/funds; no fund-loss exploit is claimed.
The draft corrects the quote rather than weakening the failing assertion. The
persisted proptest seed, deterministic input/output assertions, compatibility
property and actual release-router-WASM swap/slippage regression cover it.

Written justification is this section. CONTRIBUTING.md requires a second
reviewer for quote math: independent maintainer review remains required before
merge. This proposal is prepared for review only; it is not deployed or merged.
Stress mode honors PROPTEST_CASES with at least 2,000 accepted cases by default;
the legitimate bounded-domain rejection budget scales with case count. This
changes no assertion and avoids an unrelated 100k-case reject-budget abort.

## 5. GitHub Actions: completed results, no pending jobs

Both inspected workflows completed; no pending, queued or cancelled checks:

| Job | PR head e4351d7 | Merged main 4bb3d09 |
|---|---|---|
| Build WASM and run tests (includes fmt, clippy, artifacts/rebuild) | PASS | PASS |
| Build TypeScript bindings (factory), including npm audit | PASS | PASS |
| Build TypeScript bindings (pair), including npm audit | PASS | PASS |
| Build TypeScript bindings (router), including npm audit | PASS | PASS |
| Rust dependency advisories (both lockfiles) | FAIL | FAIL |

[PR workflow](https://github.com/VELTRIXAE/Contract/actions/runs/41838897943).
[Post-merge workflow](https://github.com/VELTRIXAE/Contract/actions/runs/41844765358).
The failed job successfully installed cargo-audit; root audit exited 1 with one
vulnerability and two unmaintained warnings. The generator audit still ran and
passed (121 dependencies). This was an actual dependency failure, not a tool
installation failure or silently skipped check. Test-helper deprecation warnings
remain. Npm jobs reported zero vulnerabilities.

The observed merge despite a failed audit warrants maintainer/admin review of
required checks and bypass policy. This review has not determined the protection
configuration or modified it. Follow-up branch CI must be assessed separately;
old PR #9 results are not presented as results for new math changes.

## 6. Follow-up verification and remaining approvals

Final local follow-up results:

| Command/check | Status / output |
|---|---|
| `make build` | PASS, exit 0 |
| `make test` | PASS, exit 0: **269 passed, zero failed/ignored** |
| `make lint` | PASS, exit 0; test-helper deprecation warnings remain |
| `make fmt-check` | PASS, exit 0 |
| `make bindings metadata` | PASS, exit 0; callable specs/client source unchanged |
| Fresh independent-target release build / byte comparison | PASS; all four artifacts match; only router differs from PR #9 |
| `PROPTEST_CASES=100000 cargo test --locked -p veltrixae-library --test amm_properties` | PASS; nine randomized properties at 100k accepted cases each, one deterministic regression |
| `python3 -m unittest discover -s scripts/tests -v` | PASS; **12 tests**, all HTTP/CLI calls mocked |
| Secret-pattern scan / `git diff --check` / shell syntax | PASS; no recognized credentials; no whitespace/syntax failures |
| Workspace RustSec | FAIL, unchanged RUSTSEC-2026-0387 plus derivative/paste warnings; gates retained |
| Deployment/initialization/restore | NOT RUN; no approval or authentic IDs |

Individual outcomes: [test evidence](evidence/pr9-remediation-test-results.json).
Current hashes: [WASM hashes](evidence/pr9-remediation-wasm.sha256). Original PR
#9 evidence remains historical rather than being silently rewritten. Raw logs
stay outside the commit in `/tmp/veltrixae-pr9-review-*`.

Suite totals: factory 66, library unit 23, property/regression 10, pair 64,
router 99, test token 7; total 269. No existing
assertion was removed. Follow-up CI also runs the validated 100k-case property
command as a separate required-to-pass job step. The verifier now fails unavailable/malformed pair counts
and its fixtures intercept every HTTP/CLI call.

Still required: independent math review; approved cryptography-stack remediation;
rightsholder/legal licensing resolution; trusted deployment evidence and explicit
human approval before any on-chain change. No credentials requested, no
advisory ignored, no security gate disabled, and no automatic merge performed.
