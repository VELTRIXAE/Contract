# VELTRIXAE contract bindings

Generated TypeScript clients for **veltrixae-frontend** and
**veltrixae-aggregator**.

| Package             | Contract          | Source WASM                  |
|---------------------|-------------------|------------------------------|
| `@veltrixae/factory`| VELTRIXAE Factory | `veltrixae_factory.wasm`     |
| `@veltrixae/pair`   | VELTRIXAE Pair    | `veltrixae_pair.wasm`        |
| `@veltrixae/router` | VELTRIXAE Router  | `veltrixae_router.wasm`      |

Do not edit `src/index.ts` by hand. Regenerate after any contract interface
change:

```bash
make bindings          # or ./scripts/generate-bindings.sh
```

The generator is `tools/spec-tool`, which wraps `soroban-spec-typescript`
(the same code behind `stellar contract bindings typescript`).

No contract ID or network is baked in. Pass them at runtime:

```ts
import { Client } from "@veltrixae/router";

const router = new Client({
  contractId: process.env.VELTRIXAE_ROUTER_ID!, // from deployments/testnet.json
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
  publicKey,           // the user's G... address
  signTransaction,     // from the user's wallet
});

const tx = await router.swap_exact_tokens_for_tokens({
  amount_in: 10_000_000n,
  amount_out_min: 9_900_000n,          // slippage protection
  path: [tokenIn, tokenOut],
  to: publicKey,
  deadline: BigInt(Math.floor(Date.now() / 1000) + 300),
});
const { result } = await tx.signAndSend();
```

Build a package locally:

```bash
cd bindings/router && npm ci && npm run build
```
