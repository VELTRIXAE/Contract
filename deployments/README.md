# Deployments

`deploy-testnet.sh` writes `<network>.json` here **after** an authorized
deployment, with the real contract IDs, WASM hashes and transaction hashes
returned by the network. `create-pool.sh` appends created pools.

No deployment record is present in the available repository/history, so there
is no `testnet.json`. This does not establish that no deployment exists elsewhere. Do not hand-edit or invent IDs: every value in these files must
come from an actual network response.

Mainnet deployments are out of scope for these scripts and are refused.
