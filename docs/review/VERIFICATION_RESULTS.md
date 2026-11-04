# Final verification results

Historical snapshot for PR #9 head `e4351d7`; follow-up findings and current
verification are in [PR9_REMEDIATION.md](PR9_REMEDIATION.md).

2026-11-04; baseline `499fef4`, branch `funding-readiness/contracts` with atomic
factory/router constructors and the changes summarized in PR_SUMMARY.md.
This is reproducible engineering evidence, not an independent security audit.

Environment: Rust 1.99.1 (58cf8f9c3), Cargo 1.99.1, Linux x86_64 host,
wasm32v1-none target; Node v24.22.2, npm 11.19.1 locally (CI uses Node 22).
Security tool: cargo-audit 0.22.2; RustSec DB
`965bf9ad083b58e6340d65829d85217b18be9290`, 1403 advisories.

Raw execution/debug logs are local in `/tmp/veltrixae-funding-review/` and
`/tmp/veltrixae-followup-*.log`; they are excluded from commits. This document
preserves the relevant observed output and exit codes. Each of the 264 individual
Rust test outcomes is saved in [test-results.json](evidence/test-results.json).
No generated WASM, node_modules, dist, keys, credentials or test snapshots are
included in the commit. Generated client source/interface metadata are intentional
tracked artifacts and were regenerated for the constructor spec change.

| Command/check | Result | Exit / observed output |
|---|---|---|
| `make build` | PASS | 0; `Finished release profile [optimized]` |
| `make test` | PASS | 0; **264 passed, 0 failed**, no ignored tests |
| `make lint` | PASS | 0; correctness lints passed; test-helper deprecation warnings remain |
| `make fmt-check` | PASS | 0; no formatting diff |
| `make bindings metadata` | PASS | 0; regenerated all three clients and metadata |
| Second generation / compare files by SHA-256 | PASS | 0; identical client source/metadata output |
| Fresh independent-target release build and compare | PASS | 0; four byte-identical WASM files |
| `npm ci --no-audit --no-fund --prefix bindings/factory` then `npm run build --prefix bindings/factory` | PASS | Both 0; 45 packages installed, `tsc` succeeds |
| Same npm install/build for `bindings/pair` | PASS | Both 0; `tsc` succeeds |
| Same npm install/build for `bindings/router` | PASS | Both 0; `tsc` succeeds |
| `npm audit --prefix bindings/factory --audit-level=high` | PASS | 0; `found 0 vulnerabilities` |
| Same npm audit for `bindings/pair` | PASS | 0; `found 0 vulnerabilities` |
| Same npm audit for `bindings/router` | PASS | 0; `found 0 vulnerabilities` |
| `cargo audit --file Cargo.lock` | **FAIL** | 1; `error: 1 vulnerability found!`, 2 unmaintained warnings |
| `cargo audit --file tools/spec-tool/Cargo.lock` | PASS | 0; 121 dependencies scanned, no advisories |
| `cargo update --offline --dry-run -p crypto-bigint --precise 0.7.10` | FAIL (compatibility experiment) | 101; 0.7.10 cannot satisfy `crypto-bigint = "^0.5"` required by elliptic-curve 0.13.8; lockfile unchanged |
| `python3 -m unittest discover -s scripts/tests -v` | PASS | 0; `Ran 9 tests`, `OK` |
| `python3 scripts/check-secrets.py` | PASS | 0; no recognized credential patterns, values suppressed |
| `python3 scripts/check-secrets.py --history` | PASS | 0; all reachable historical blobs scanned before commit, no recognized patterns |
| `bash -n` for every `scripts/*.sh` | PASS | 0; no output |
| `git diff --check` | PASS | 0; no output |
| Preserved licensing / lockfiles | PASS | No diff to LICENSE, LICENSE-APACHE, NOTICE or either Cargo.lock |
| Public Testnet identity (captured read-only) | PASS | Horizon/RPC passphrase `Test SDF Network ; September 2015`; [Horizon](evidence/horizon-root.json), [RPC](evidence/rpc-network.json) |
| Project Testnet IDs, receipts, code and state | **NOT RUN / NOT VERIFIED** | No authentic deployment record available; maintainer confirmed |
| Deploy/initialize/pool/TTL/restore transactions | NOT RUN | Not authorized; none submitted |
| `make optimize` / live Stellar CLI constructor deployment | NOT RUN | CLI unavailable; SDK release-WASM deployment tests pass |
| Remote PR workflow | Separate from local results | Observe actual PR checks; workspace advisory gate is expected to fail until dependency remediation |

## Individual suites

| Suite | Passed | Failed |
|---|---:|---:|
| Factory | 65 | 0 |
| Factory interface | 0 | 0 |
| Library unit | 23 | 0 |
| Library property | 8 | 0 |
| Pair | 64 | 0 |
| Router | 97 | 0 |
| Test token | 7 | 0 |
| **Total** | **264** | **0** |

The initial repository baseline had 253 passing tests, the earlier maintainer
review had 260, and this follow-up adds four release-WASM constructor deployment
regressions. All existing assertions remain. Native negative/pre-initialization
fixtures deliberately clear constructor state; those artificial legacy states
are not reachable through new production deployments.

## Reproduce the clean build comparison

Use a fresh empty output directory; do not clean a directory while another
build/test/lint process is using it:

```bash
make build
cargo build --locked --target wasm32v1-none --release --target-dir /tmp/veltrixae-funding-repro
for artifact in target/wasm32v1-none/release/veltrixae_*.wasm; do
  cmp "$artifact" "/tmp/veltrixae-funding-repro/wasm32v1-none/release/$(basename "$artifact")" || exit 1
done
sha256sum --check docs/review/evidence/wasm.sha256
```

Observed: factory, pair, router and test-token artifacts all matched exactly.
[Hashes](evidence/wasm.sha256) are local build evidence, not on-chain hashes.
This proves repeatability on the inspected host/toolchain, not cross-platform
or optimizer reproducibility. Constructor changes alter factory/router WASM;
pair and test-token hashes retain their previous values.

## Unresolved security result

RUSTSEC-2026-0387 affects locked crypto-bigint 0.5.5 in the SDK host/test chain;
patched >=0.7.6 does not satisfy the current elliptic-curve 0.5 requirement.
No safe lockfile-only fix was found. The normal WASM graph excludes the crate,
but that does not turn the repository audit green. The gate remains enabled.
Warnings: derivative/RUSTSEC-2024-0388, paste/RUSTSEC-2024-0436 (unmaintained).
See [dependency investigation](../DEPENDENCY_SECURITY.md),
[licensing questions](../LICENSING_REVIEW.md) and
[Testnet verification](../TESTNET_VERIFICATION.md) for remaining blockers.
