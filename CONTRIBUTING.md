# Contributing to VELTRIXAE Contracts

## Setup

```bash
git clone https://github.com/VELTRIXAE/Contract && cd Contract
make test   # rustup installs Rust 1.99.1 and wasm32v1-none on first run
```

## Workflow

1. Branch from `main` (`feat/...`, `fix/...`, `docs/...`).
2. Make the change **and** its tests. Run `make fmt-check`, `make test` and `make lint`.
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

## Review evidence

Include the base commit, `rustc --version`, `cargo --version`, command exit
codes, and relevant output in the PR. Label checks PASS, FAIL or NOT RUN; explain
failures and missing tools. Deprecation warnings are not proof of failure, and
an absent deployment record is not proof that no contract exists elsewhere.
Use the committed lockfiles (`--locked`) in both Cargo workspaces and `npm ci`
for all three binding packages. Dependency advisory checks must not be bypassed:

```bash
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-secrets.py
cargo audit --file Cargo.lock
cargo audit --file tools/spec-tool/Cargo.lock
npm audit --prefix bindings/factory --audit-level=high
npm audit --prefix bindings/pair --audit-level=high
npm audit --prefix bindings/router --audit-level=high
```

Compare regenerated metadata against compiled artifacts. For release candidates,
record WASM hashes, do a clean rebuild with the same toolchain, and compare hashes
again. Optimized artifacts additionally require the exact CLI/optimizer version;
a release build match alone does not establish optimizer reproducibility.

Tests should assert state after failure, including nested transfers, LP balances,
reserves and registry entries. Use negative authorization tests without blanket
mocks. Do not call a mocked-auth test signed-envelope replay coverage. Storage
changes need ledger-advance/TTL tests and a separate restoration plan.

Before any Testnet write, obtain explicit human approval of the concrete plan.
A contributor must never infer approval from a review request. Read-only queries
must use simulation (`--send=no`) because getter TTL changes can otherwise persist.
Recover and verify existing deployments before proposing new ones. The scripts
pin public Testnet; local/mainnet aliases are refused. Never paste seeds into
logs, issues or PRs; the heuristic scanner supplements human review.

See [contributor issue drafts](docs/issues/README.md) for scoped work. Keep
Apache/NOTICE/original author information, distinguish VELTRIXAE changes from
Soroswap/Uniswap mathematics, and flag the unresolved library license conflict.
Security-sensitive changes need a maintainer review; AMM changes additionally
need the written justification and second reviewer required above.
