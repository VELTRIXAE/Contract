# Contributing to VELTRIXAE Contracts

## Setup

```bash
git clone https://github.com/VELTRIXAE/Contract && cd Contract
make test   # installs the pinned toolchain via rustup on first run
```

## Workflow

1. Branch from `main` (`feat/...`, `fix/...`, `docs/...`).
2. Make the change **and** its tests. Run `make test` and `make lint`.
3. If you changed a public function, argument, return type, error or event:
   - run `make bindings metadata` and commit the regenerated files,
   - update `docs/CONTRACTS.md`,
   - call out the interface change in the PR (frontend and aggregator depend
     on it).
4. If you changed contract behaviour, add an entry to
   `docs/UPSTREAM_ATTRIBUTION.md` (modifications list) and, for anything
   security-relevant, to `SECURITY.md`.
5. Open a PR. CI must pass.

Commit messages follow Conventional Commits (`feat:`, `fix:`, `test:`,
`docs:`, `build:`, `ci:`, `deps:`).

## Rules for contract code

- **Do not change AMM math** (`library/src/quotes.rs`, pair `deposit`,
  `swap`, `withdraw`, `mint_fee`) without a written justification, new tests,
  and a second reviewer.
- Use `checked_*` arithmetic; never rely on wrapping. The release profile
  has `overflow-checks = true` as a backstop, not as the primary defence.
- Keep function names, argument types, error codes and event topics stable.
  Add new error codes instead of renumbering.
- Every state-changing router function must `require_auth` the account whose
  funds move.
- Extend TTLs the same way the surrounding code does.
- Keep upstream Apache-2.0 notices. Do not remove attribution.

## Secrets and deployments

- Never commit keys, seed phrases or `.env` files.
- Never deploy from CI or automatically. Deployments are run by a person
  with `scripts/deploy-testnet.sh`, and only real network output goes into
  `deployments/`.
- Never deploy to mainnet from this repository.

## Dependencies

Dependabot opens weekly PRs. Soroban SDK crates are grouped; a major SDK
bump changes the host interface and needs a careful review, a full
`make test`, and regenerated bindings/metadata.
