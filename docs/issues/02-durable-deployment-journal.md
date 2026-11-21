# Persist partial deployment provenance and structured receipts

Suggested labels: tooling, advanced tooling

## Problem

`scripts/deploy-testnet.sh` stores intermediate logs in a temporary directory, deletes them on exit, and writes the final manifest only after all steps succeed. A failure after upload or deployment can therefore lose useful public provenance. Receipt hashes are currently extracted heuristically from logs (e.g. taking the last hexadecimal string), which can misidentify unrelated values.

## Scope

Add an atomic, public-only journal of successful steps and validated receipts. Preserve the existing final manifest schema and the existing authorization/network guards. Provide a dry-run recovery view that reports completed versus missing steps without submitting transactions.

Structured receipt validation must reject unrelated operations or mismatched contract/code identities; an arbitrary last hexadecimal string must not be accepted as a receipt identifier.

Journals must contain no signing material, CLI-keystore contents, or secret environment data.

Automatic transaction resumption (re-submitting or continuing transactions) is explicitly **out of scope** for this task. Any future resume/write mode needs its own reviewed scope.

## Acceptance criteria

- [ ] Offline mocked-CLI tests interrupt after each successful upload/deployment and verify that completed IDs, artifact hashes, and receipt identifiers remain recoverable.
- [ ] Journal writes are atomic; malformed or stale records fail clearly, and existing final manifests cannot be overwritten.
- [ ] Structured receipt validation rejects unrelated operations or mismatched contract/code identities; do not accept an arbitrary last hexadecimal string.
- [ ] Journals contain no signing material, CLI-keystore contents, or secret environment data.
- [ ] A recovery command/report identifies completed versus missing steps without submitting transactions.
- [ ] Deployment documentation is updated with journal location, schema, and the manual recovery procedure.

## Out of scope

- Automatic transaction resumption or any write/resume mode that submits transactions.
- Public-network deployment or funding. This issue authorizes no public-network deployment or funding.

## Contributor guidance

Estimated difficulty: advanced tooling; not a good-first-issue. Start with `scripts/deploy-testnet.sh`, `scripts/lib.sh`, and `scripts/tests/test_guards.py`. Follow `CONTRIBUTING.md`, preserve attribution, and exercise the scripts through offline mocks.