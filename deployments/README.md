# Deployments

`deploy-testnet.sh` writes `<network>.json` here **after** an authorized
deployment, with the real contract IDs, WASM hashes and transaction hashes
returned by the network. `create-pool.sh` appends created pools.

No deployment has been made from this repository yet, so there is no
`testnet.json`. Do not hand-edit or invent IDs: every value in these files must
come from an actual network response.

Mainnet deployments are out of scope for these scripts and are refused.
