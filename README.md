# VELTRIXAE Contracts

Soroban smart contracts for the **VELTRIXAE** constant-product AMM on Stellar.

| Contract              | Crate                 | WASM                     | Role                                                |
|-----------------------|-----------------------|--------------------------|-----------------------------------------------------|
| **VELTRIXAE Factory** | `veltrixae-factory`   | `veltrixae_factory.wasm` | Creates, registers and indexes pairs; fee settings  |
| **VELTRIXAE Pair**    | `veltrixae-pair`      | `veltrixae_pair.wasm`    | One pool per token pair: reserves, swaps, LP token  |
| **VELTRIXAE Router**  | `veltrixae-router`    | `veltrixae_router.wasm`  | User entry point: swaps, multi-hop, add/remove liquidity |

Supporting crates: `veltrixae-library` (pair address derivation and quote
math), `veltrixae-factory-interface` (shared factory trait, client and errors)
and `veltrixae-test-token` (test-only token).

> **Upstream.** These contracts are derived from
> [Soroswap Core](https://github.com/soroswap/core) (Apache-2.0), commit
> `6eade00`. The constant-product invariant, fee policy and LP accounting follow upstream;
> exact-output quotes include a documented integer-rounding correction. See
> [docs/UPSTREAM_ATTRIBUTION.md](docs/UPSTREAM_ATTRIBUTION.md) for every
> modification.

> **Status.** Testnet only. **Not audited.** The upstream Soroswap code was
> audited by OtterSec in 2024; that audit does not cover the VELTRIXAE changes
> or the SDK upgrade. Read [SECURITY.md](SECURITY.md) before using these
> contracts with real value.

## Quick start

Requirements: Rust 1.99.1 (installed automatically from `rust-toolchain.toml`,
including the `wasm32v1-none` target), `make`. Optional: the
[Stellar CLI](https://developers.stellar.org/docs/tools/cli) for optimization
and deployment, Node.js for building the bindings, `jq` for the scripts.

```bash
make build      # compile all contracts to target/wasm32v1-none/release/
make test       # build WASM, then run every unit, integration and property test
make lint       # clippy correctness lints
make bindings   # regenerate TypeScript clients in bindings/
make metadata   # regenerate interface JSON in metadata/
```

Integration tests load the compiled WASM, so always use `make test` (or build
the WASM first) after changing contract code.

## Repository layout

```
contracts/
  factory/            VELTRIXAE Factory
  factory-interface/  shared factory trait, client, FactoryError
  pair/               VELTRIXAE Pair (+ embedded LP token)
  router/             VELTRIXAE Router
  library/            pair_for, reserves, quote / amount math (+ property tests)
  token/              test token used by the test suites
bindings/             generated TypeScript clients (@veltrixae/factory|pair|router)
metadata/             machine-readable contract interface (JSON)
scripts/              build, testnet deploy, pool creation, verification
deployments/          real deployment records (written by the scripts only)
tools/spec-tool/      offline bindings / metadata generator
docs/                 architecture, contract reference, deployment, testing
```

## Using the contracts

Integrators should talk to the **Router** only; the pair has no access control
on `deposit`, `swap` and `withdraw` and expects tokens to have been
transferred to it in the same transaction. The full public interface,
errors and events are in [docs/CONTRACTS.md](docs/CONTRACTS.md).

New factory/router deployments initialize atomically in constructors; see
[deployment arguments](docs/CONTRACTS.md#deployment-initialization). Existing
immutable deployments retain their original behavior.

Compatibility: existing callable function names, argument types, error codes and event topics
are identical to Soroswap Core, so existing Soroswap-compatible frontends,
aggregators and indexers work against VELTRIXAE deployments by swapping
contract IDs.

## Deployment

Testnet deployment is scripted but **never automatic**: every script that
submits transactions requires an explicit confirmation, refuses mainnet, and
reads keys only from the Stellar CLI keystore. See
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

Successful deployment steps are recorded in an atomic, public-only journal
under `deployments/` (completed IDs, artifact hashes and validated receipt
identifiers), so a failure part-way through no longer discards provenance. The
journal contains no signing material, keystore contents or secret environment
data, and the final manifest is only written after all steps succeed. A
read-only recovery report lists completed versus missing steps without
submitting transactions; automatic resumption is not implemented. Journal
location, schema and the manual recovery procedure are documented in
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

An older Testnet deployment from commit `133b18d00505108c26382cb44045abab48b51882`
has public IDs and receipts preserved in the backend repository. Read-only checks
on 2026-11-21 matched its Factory, Router and uploaded Pair WASM hashes. Its
factory has zero pools. This is **not a deployment of current main**; the newer
constructor and rounding fixes are not present in those immutable contracts.
Reconciliation of the backend-reported November 2 Factory/Router deployment
against Testnet (receipts, WASM hashes and source revision) is tracked in
[docs/TESTNET_VERIFICATION.md](docs/TESTNET_VERIFICATION.md); no manifest is
published from that evidence unless verification succeeds, and mocked evidence
is never treated as success. See that document for evidence and remaining
provenance, licensing, liquidity and wallet-signing limitations.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the contracts fit together
- [docs/CONTRACTS.md](docs/CONTRACTS.md): public methods, errors, events
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md): testnet build, deploy, verify
- [docs/TESTING.md](docs/TESTING.md): test suites and how to run them
- [docs/UPSTREAM_ATTRIBUTION.md](docs/UPSTREAM_ATTRIBUTION.md): what came from Soroswap and what changed
- [docs/TECHNICAL_AUDIT.md](docs/TECHNICAL_AUDIT.md): verified build/test status and audit findings
- [docs/TESTNET_VERIFICATION.md](docs/TESTNET_VERIFICATION.md): read-only evidence and verification blockers
- [docs/UPSTREAM_COMPARISON.md](docs/UPSTREAM_COMPARISON.md): exact upstream comparison and licensing limits
- [docs/DEPENDENCY_SECURITY.md](docs/DEPENDENCY_SECURITY.md): unresolved RustSec advisory and dependency constraints
- [docs/LICENSING_REVIEW.md](docs/LICENSING_REVIEW.md): preserved licenses and unresolved upstream conflict
- [docs/PRODUCTION_GOVERNANCE.md](docs/PRODUCTION_GOVERNANCE.md): launch gates, multisig defaults, custody policy and external approvals
- [docs/review/PR9_REMEDIATION.md](docs/review/PR9_REMEDIATION.md): PR #9 findings, verified fixes and remaining approvals
- [docs/review/BASELINE_AUDIT.md](docs/review/BASELINE_AUDIT.md): maintainer review baseline
- [SECURITY.md](SECURITY.md): threat model, known limitations, reporting
- [CONTRIBUTING.md](CONTRIBUTING.md): development workflow
- [CHANGELOG.md](CHANGELOG.md): notable changes

## License

VELTRIXAE-only additions use the [MIT License](LICENSE) where applicable.
Derived contract manifests declare [Apache License 2.0](LICENSE-APACHE) and
retain [NOTICE](NOTICE). The library also retains explicit GPL-3.0 notices;
its applicable licensing remains [unresolved](docs/LICENSING_REVIEW.md).
These declarations do not establish an unambiguous grant for that library.
