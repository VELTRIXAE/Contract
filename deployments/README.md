# Deployments

`deploy-testnet.sh` writes `<network>.json` here **after** an authorized
deployment, with the real contract IDs, WASM hashes and transaction hashes
returned by the network. `create-pool.sh` appends created pools.

No deployment record is present in the available repository/history, so there
is no `testnet.json`. This does not establish that no deployment exists elsewhere. Do not hand-edit or invent IDs: every value in these files must
come from an actual network response.

The older deployment from source commit `133b18d00505108c26382cb44045abab48b51882`
is recorded in the backend repository, with fresh read-only verification linked
from [TESTNET_VERIFICATION.md](../docs/TESTNET_VERIFICATION.md). Those legacy
records must not be copied to this operational directory: they describe an
initialize-based deployment, not the current constructor-based scripts.

Mainnet deployments are out of scope for these scripts and are refused.

## Durable deployment journal

In addition to the final manifest, `deploy-testnet.sh` maintains an atomic,
public-only journal of each successful step under `deployments/journal/`
(e.g. `deployments/journal/<network>.journal.json`). The journal is written
incrementally as steps complete, so a failure after an upload or deployment
still leaves recoverable public provenance instead of losing it with a
temporary directory.

### Journal schema

The journal is a JSON document containing only public data:

- `network`: the target network name (e.g. `testnet`).
- `steps`: an ordered list of completed steps, each with:
  - `name`: the step identifier (e.g. `upload`, `deploy`).
  - `status`: `completed`.
  - `artifact_hash`: the WASM/artifact hash for the step, when applicable.
  - `contract_id`: the deployed contract ID, when applicable.
  - `receipt`: a structured, validated receipt record (see below).
  - `timestamp`: when the step completed.

Receipts are validated structurally: the journal only records a receipt when
it corresponds to the expected operation and matches the expected
contract/code identity. An arbitrary trailing hexadecimal string from a log is
never accepted as a receipt.

The journal **never** contains signing material, CLI-keystore contents, secret
keys, or secret environment data. Only public identifiers, hashes and
validated receipts are stored.

### Atomicity and immutability

Journal writes are atomic (written to a temporary file and renamed into
place). Malformed or stale journal records cause the script to fail clearly
rather than silently continuing. The final `<network>.json` manifest is
written only after **all** steps succeed, and an existing final manifest is
never overwritten.

### Manual recovery procedure

Automatic transaction resumption is out of scope. To inspect what completed
and what is missing without submitting any transactions, run the dry-run
recovery report:

```sh
scripts/deploy-testnet.sh --recover --network testnet
```

This reads the journal and prints a report identifying completed versus
missing steps, including the recorded contract IDs, artifact hashes and
receipt identifiers. It performs no network writes and submits no
transactions. Use this report to determine which steps remain before
re-running an authorized deployment; any future resume/write mode requires
its own reviewed scope.