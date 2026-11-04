# Validate constructor deployment envelopes and legacy migration guidance

Suggested labels: security, tooling

## Problem

Factory/router now initialize atomically in SDK 22 constructors. Local release
WASM tests cover takeover rejection and missing-argument rollback, but no live
CLI deployment has been executed and existing immutable deployments are unknown.

## Scope

Add offline transaction-envelope tests using CLI build-only output for both
constructor deployments, checking arguments, source, Testnet domain and atomic
creation. Document identification of legacy bytecode versus constructor-enabled
artifacts, and an operator-reviewed migration plan. Never initialize or redeploy
unknown contracts automatically; no public transaction is part of this issue.

## Acceptance criteria

Decode actual build-only XDR and assert CreateContractV2 constructor argument
order/types; missing arguments fail before submission; factory/router state
matches intended values in a host execution; legacy risks and immutable-code
limits are explicit. Maintain AMM formulas, function signatures and notices.
