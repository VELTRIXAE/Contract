# Persist partial deployment provenance and structured receipts

Suggested labels: tooling, good first issue

## Problem

Deployment IDs/receipts are only committed to the final manifest after all steps succeed; temporary logs disappear on failure, and a last-hex heuristic can misidentify hashes.

## Scope

Persist a public-only journal atomically after each successful step; validate structured CLI/RPC receipts and operation-to-ID/code bindings; refuse stale optimized binaries and record optimizer provenance.

## Acceptance criteria

Offline interruption tests recover every completed ID/receipt without redeployment; no signing material is logged; repeated resume skips completed work; XDR provenance checks reject unrelated successful transactions.
