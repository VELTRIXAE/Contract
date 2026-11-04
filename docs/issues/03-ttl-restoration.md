# Exercise archival and restoration across registry and LP balances

Suggested labels: testing, soroban

## Problem

TTL targets differ for instances, factory registry and LP balances. Activity does not refresh all inactive records. A renewal unit test does not exercise public-network restore behavior.

## Scope

Add host archival/restore tests for instance/code (including uploaded pair WASM before any pool exists), pair lookup/index entries and LP balances; document explicit operator-approved restoration steps.

## Acceptance criteria

Preserve reserves, total supply, pair identity and authority after restoration; demonstrate expired temporary allowances stay expired; document ledger-count versus time assumptions. No automatic TTL transactions.
