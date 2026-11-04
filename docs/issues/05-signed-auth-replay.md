# Test signed Soroban authorization replay and expiration

Suggested labels: security, testing

## Problem

Current tests exercise missing authorization, auth trees and initialization replay, but do not establish signed nonce replay rejection.

## Scope

Add a host integration harness using test-only generated signing keys and actual authorization entries for a router call; reuse authorization and vary nonce, network domain and expiration.

## Acceptance criteria

First authorized call succeeds; reuse, wrong network and expired signatures fail without balance/reserve/LP changes. Keys stay ephemeral; no public network transaction is needed.
