# Migrate deprecated SDK test helpers without changing semantics

Suggested labels: testing, good first issue

## Problem

Current test runs emit deprecation warnings for SDK registration, asset and budget helpers. Production artifacts are unaffected.

## Scope

Replace deprecated test APIs in small crate-scoped changes; explain any auth/event or registration differences and compare counts and assertions.

## Acceptance criteria

All existing and new tests pass with no weakened assertions; deprecation warnings from repository helpers are removed; compiled production contract hashes stay identical.
