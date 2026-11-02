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
> `6eade00`. The AMM math is unchanged. See
> [docs/UPSTREAM_ATTRIBUTION.md](docs/UPSTREAM_ATTRIBUTION.md) for every
> modification.

> **Status.** Testnet only. **Not audited.** The upstream Soroswap code was
> audited by OtterSec in 2024; that audit does not cover the VELTRIXAE changes
> or the SDK upgrade. Read [SECURITY.md](SECURITY.md) before using these
> contracts with real value.

## Quick start

Requirements: Rust stable (installed automatically from `rust-toolchain.toml`,
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

Compatibility: function names, argument types, error codes and event topics
are identical to Soroswap Core, so existing Soroswap-compatible frontends,
aggregators and indexers work against VELTRIXAE deployments by swapping
contract IDs.

## Deployment

Testnet deployment is scripted but **never automatic**: every script that
submits transactions requires an explicit confirmation, refuses mainnet, and
reads keys only from the Stellar CLI keystore. See
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

No VELTRIXAE contracts have been deployed from this repository yet.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the contracts fit together
- [docs/CONTRACTS.md](docs/CONTRACTS.md): public methods, errors, events
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md): testnet build, deploy, verify
- [docs/TESTING.md](docs/TESTING.md): test suites and how to run them
- [docs/UPSTREAM_ATTRIBUTION.md](docs/UPSTREAM_ATTRIBUTION.md): what came from Soroswap and what changed
- [docs/TECHNICAL_AUDIT.md](docs/TECHNICAL_AUDIT.md): verified build/test status and audit findings
- [SECURITY.md](SECURITY.md): threat model, known limitations, reporting
- [CONTRIBUTING.md](CONTRIBUTING.md): development workflow
- [CHANGELOG.md](CHANGELOG.md): notable changes

## License

VELTRIXAE-authored code is under the [MIT License](LICENSE). Code derived from
Soroswap Core remains under the [Apache License 2.0](LICENSE-APACHE); see
[NOTICE](NOTICE). The crate manifests declare `Apache-2.0`, the license of the
derived contract code.
