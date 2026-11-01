import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const SoroswapPairError = {
  /**
   * SoroswapPair: already initialized
   */
  101: {message:"InitializeAlreadyInitialized"},
  /**
   * SoroswapPair: not yet initialized
   */
  102: {message:"NotInitialized"},
  /**
   * SoroswapPair: token_0 must be less than token_1 while initializing
   */
  103: {message:"InitializeTokenOrderInvalid"},
  /**
   * SoroswapPair: insufficient amount of token 0 sent while doing deposit
   */
  104: {message:"DepositInsufficientAmountToken0"},
  /**
   * SoroswapPair: insufficient amount of token 1 sent while doing deposit
   */
  105: {message:"DepositInsufficientAmountToken1"},
  /**
   * SoroswapPair: insufficient first liquidity minted while doing deposit
   */
  106: {message:"DepositInsufficientFirstLiquidity"},
  /**
   * SoroswapPair: insufficient liquidity minted while doing deposit
   */
  107: {message:"DepositInsufficientLiquidityMinted"},
  /**
   * SoroswapPair: insufficient output amount while doing deposDepositit
   */
  108: {message:"SwapInsufficientOutputAmount"},
  /**
   * SoroswapPair: negatives amounts out dont supported while doing swap
   */
  109: {message:"SwapNegativesOutNotSupported"},
  /**
   * SoroswapPair: insufficient liquidity to do the swap
   */
  110: {message:"SwapInsufficientLiquidity"},
  /**
   * SoroswapPair: invalid to to do the swap
   */
  111: {message:"SwapInvalidTo"},
  /**
   * SoroswapPair: insufficient input amount while doing swap
   */
  112: {message:"SwapInsufficientInputAmount"},
  /**
   * SoroswapPair: negatives amounts in dont supported while doing swap
   */
  113: {message:"SwapNegativesInNotSupported"},
  /**
   * SoroswapPair: K constant is not met while doing swap
   */
  114: {message:"SwapKConstantNotMet"},
  /**
   * SoroswapPair: liquidity was not initialized yet while doing withdraw
   */
  115: {message:"WithdrawLiquidityNotInitialized"},
  /**
   * SoroswapPair: insufficient sent shares while doing withdraw
   */
  116: {message:"WithdrawInsufficientSentShares"},
  /**
   * SoroswapPair: insufficient liquidity burned while doing withdraw
   */
  117: {message:"WithdrawInsufficientLiquidityBurned"},
  /**
   * SoroswapPair: OVERFLOW while updating
   */
  118: {message:"UpdateOverflow"}
}


export interface SkimEvent {
  skimmed_0: i128;
  skimmed_1: i128;
}


export interface SwapEvent {
  amount_0_in: i128;
  amount_0_out: i128;
  amount_1_in: i128;
  amount_1_out: i128;
  to: string;
}


export interface SyncEvent {
  new_reserve_0: i128;
  new_reserve_1: i128;
}


export interface DepositEvent {
  amount_0: i128;
  amount_1: i128;
  liquidity: i128;
  new_reserve_0: i128;
  new_reserve_1: i128;
  to: string;
}


export interface WithdrawEvent {
  amount_0: i128;
  amount_1: i128;
  liquidity: i128;
  new_reserve_0: i128;
  new_reserve_1: i128;
  to: string;
}

export type DataKey = {tag: "Allowance", values: readonly [AllowanceDataKey]} | {tag: "Balance", values: readonly [string]} | {tag: "TotalSupply", values: void};


export interface AllowanceValue {
  amount: i128;
  expiration_ledger: u32;
}


export interface AllowanceDataKey {
  from: string;
  spender: string;
}


export interface TokenMetadata {
  decimal: u32;
  name: string;
  symbol: string;
}

export const FactoryError = {
  /**
   * SoroswapFactory: not yet initialized
   */
  201: {message:"NotInitialized"},
  /**
   * SoroswapFactory: token_a and token_b have identical addresses
   */
  202: {message:"CreatePairIdenticalTokens"},
  /**
   * SoroswapFactory: pair already exists between token_a and token_b
   */
  203: {message:"CreatePairAlreadyExists"},
  /**
   * SoroswapFactory: already initialized
   */
  204: {message:"InitializeAlreadyInitialized"},
  /**
   * SoroswapFactory: pair does not exist
   */
  205: {message:"PairDoesNotExist"},
  /**
   * SoroswapFactory: index does not exist
   */
  206: {message:"IndexDoesNotExist"}
}

export interface Client {
  /**
   * Construct and simulate a skim transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Skims excess tokens from reserves and sends them to the specified address.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * * `to` - The address where the excess tokens will be sent.
   */
  skim: ({to}: {to: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Executes a token swap within the Soroswap pair.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * * `amount_0_out` - The desired amount of the first token to receive.
   * * `amount_1_out` - The desired amount of the second token to receive.
   * * `to` - The address where the swapped tokens will be sent.
   * Returns an error if the swap cannot be executed. Possible errors include:
   * - `SoroswapPairError::NotInitialized`
   * - `SoroswapPairError::SwapInsufficientOutputAmount`
   * - `SoroswapPairError::SwapNegativesOutNotSupported`
   * - `SoroswapPairError::SwapInsufficientLiquidity`
   * - `SoroswapPairError::SwapInvalidTo`
   * - `SoroswapPairError::SwapInsufficientInputAmount`
   * - `SoroswapPairError::SwapNegativesInNotSupported`
   * - `SoroswapPairError::SwapKConstantNotMet`: If the K constant condition is not met after the swap.
   */
  swap: ({amount_0_out, amount_1_out, to}: {amount_0_out: i128, amount_1_out: i128, to: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a sync transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Forces reserves to match current balances.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   */
  sync: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a k_last transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the value of the last product of reserves (`K`) stored in the contract.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * 
   * # Returns
   * The value of the last product of reserves (`K`).
   */
  k_last: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits tokens into the Soroswap pair and mints LP tokens in return.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * * `to` - The address where the minted LP tokens will be sent.
   * 
   * # Returns
   * The amount of minted LP tokens.
   * Possible errors:
   * - `SoroswapPairError::NotInitialized`: The Soroswap pair has not been initialized.
   * - `SoroswapPairError::DepositInsufficientAmountToken0`: Insufficient amount of token 0 sent.
   * - `SoroswapPairError::DepositInsufficientAmountToken1`: Insufficient amount of token 1 sent.
   * - `SoroswapPairError::DepositInsufficientFirstLiquidity`: Insufficient first liquidity minted.
   * - `SoroswapPairError::DepositInsufficientLiquidityMinted`: Insufficient liquidity minted.
   * - `SoroswapPairError::UpdateOverflow`: Overflow occurred during update.
   */
  deposit: ({to}: {to: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a factory transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the address of the Soroswap factory contract.
   */
  factory: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a token_0 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the address of the first token in the Soroswap pair.
   */
  token_0: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a token_1 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the address of the second token in the Soroswap pair.
   */
  token_1: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws liquidity from the Soroswap pair, burning LP tokens and returning the corresponding tokens to the user.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * * `to` - The address where the withdrawn tokens will be sent.
   * 
   * # Returns
   * A tuple containing the amounts of token 0 and token 1 withdrawn from the pair.
   */
  withdraw: ({to}: {to: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<readonly [i128, i128]>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes a new Soroswap pair by setting token addresses, factory, and initial reserves.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * * `factory` - The address of the Soroswap factory contract.
   * * `token_0` - The address of the first token in the pair.
   * * `token_1` - The address of the second token in the pair.
   */
  initialize: ({factory, token_0, token_1}: {factory: string, token_0: string, token_1: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_reserves transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current reserves and the last block timestamp.
   * 
   * # Arguments
   * * `e` - The runtime environment.
   * 
   * # Returns
   * A tuple containing the reserves of token 0 and token 1.
   */
  get_reserves: (options?: MethodOptions) => Promise<AssembledTransaction<readonly [i128, i128]>>

  /**
   * Construct and simulate a burn transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  burn: ({from, amount}: {from: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a name transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  name: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a symbol transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  symbol: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a approve transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  approve: ({from, spender, amount, expiration_ledger}: {from: string, spender: string, amount: i128, expiration_ledger: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  balance: ({id}: {id: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  decimals: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a transfer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer: ({from, to, amount}: {from: string, to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a allowance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  allowance: ({from, spender}: {from: string, spender: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a burn_from transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  burn_from: ({spender, from, amount}: {spender: string, from: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a total_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  total_supply: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a transfer_from transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_from: ({spender, from, to, amount}: {spender: string, from: string, to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAALNTa2ltcyBleGNlc3MgdG9rZW5zIGZyb20gcmVzZXJ2ZXMgYW5kIHNlbmRzIHRoZW0gdG8gdGhlIHNwZWNpZmllZCBhZGRyZXNzLgoKIyBBcmd1bWVudHMKKiBgZWAgLSBUaGUgcnVudGltZSBlbnZpcm9ubWVudC4KKiBgdG9gIC0gVGhlIGFkZHJlc3Mgd2hlcmUgdGhlIGV4Y2VzcyB0b2tlbnMgd2lsbCBiZSBzZW50LgAAAAAEc2tpbQAAAAEAAAAAAAAAAnRvAAAAAAATAAAAAA==",
        "AAAAAAAAAxtFeGVjdXRlcyBhIHRva2VuIHN3YXAgd2l0aGluIHRoZSBTb3Jvc3dhcCBwYWlyLgoKIyBBcmd1bWVudHMKKiBgZWAgLSBUaGUgcnVudGltZSBlbnZpcm9ubWVudC4KKiBgYW1vdW50XzBfb3V0YCAtIFRoZSBkZXNpcmVkIGFtb3VudCBvZiB0aGUgZmlyc3QgdG9rZW4gdG8gcmVjZWl2ZS4KKiBgYW1vdW50XzFfb3V0YCAtIFRoZSBkZXNpcmVkIGFtb3VudCBvZiB0aGUgc2Vjb25kIHRva2VuIHRvIHJlY2VpdmUuCiogYHRvYCAtIFRoZSBhZGRyZXNzIHdoZXJlIHRoZSBzd2FwcGVkIHRva2VucyB3aWxsIGJlIHNlbnQuClJldHVybnMgYW4gZXJyb3IgaWYgdGhlIHN3YXAgY2Fubm90IGJlIGV4ZWN1dGVkLiBQb3NzaWJsZSBlcnJvcnMgaW5jbHVkZToKLSBgU29yb3N3YXBQYWlyRXJyb3I6Ok5vdEluaXRpYWxpemVkYAotIGBTb3Jvc3dhcFBhaXJFcnJvcjo6U3dhcEluc3VmZmljaWVudE91dHB1dEFtb3VudGAKLSBgU29yb3N3YXBQYWlyRXJyb3I6OlN3YXBOZWdhdGl2ZXNPdXROb3RTdXBwb3J0ZWRgCi0gYFNvcm9zd2FwUGFpckVycm9yOjpTd2FwSW5zdWZmaWNpZW50TGlxdWlkaXR5YAotIGBTb3Jvc3dhcFBhaXJFcnJvcjo6U3dhcEludmFsaWRUb2AKLSBgU29yb3N3YXBQYWlyRXJyb3I6OlN3YXBJbnN1ZmZpY2llbnRJbnB1dEFtb3VudGAKLSBgU29yb3N3YXBQYWlyRXJyb3I6OlN3YXBOZWdhdGl2ZXNJbk5vdFN1cHBvcnRlZGAKLSBgU29yb3N3YXBQYWlyRXJyb3I6OlN3YXBLQ29uc3RhbnROb3RNZXRgOiBJZiB0aGUgSyBjb25zdGFudCBjb25kaXRpb24gaXMgbm90IG1ldCBhZnRlciB0aGUgc3dhcC4AAAAABHN3YXAAAAADAAAAAAAAAAxhbW91bnRfMF9vdXQAAAALAAAAAAAAAAxhbW91bnRfMV9vdXQAAAALAAAAAAAAAAJ0bwAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAEVNvcm9zd2FwUGFpckVycm9yAAAA",
        "AAAAAAAAAFhGb3JjZXMgcmVzZXJ2ZXMgdG8gbWF0Y2ggY3VycmVudCBiYWxhbmNlcy4KCiMgQXJndW1lbnRzCiogYGVgIC0gVGhlIHJ1bnRpbWUgZW52aXJvbm1lbnQuAAAABHN5bmMAAAAAAAAAAA==",
        "AAAAAAAAALlSZXR1cm5zIHRoZSB2YWx1ZSBvZiB0aGUgbGFzdCBwcm9kdWN0IG9mIHJlc2VydmVzIChgS2ApIHN0b3JlZCBpbiB0aGUgY29udHJhY3QuCgojIEFyZ3VtZW50cwoqIGBlYCAtIFRoZSBydW50aW1lIGVudmlyb25tZW50LgoKIyBSZXR1cm5zClRoZSB2YWx1ZSBvZiB0aGUgbGFzdCBwcm9kdWN0IG9mIHJlc2VydmVzIChgS2ApLgAAAAAAAAZrX2xhc3QAAAAAAAAAAAABAAAACw==",
        "AAAAAAAAAvtEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgU29yb3N3YXAgcGFpciBhbmQgbWludHMgTFAgdG9rZW5zIGluIHJldHVybi4KCiMgQXJndW1lbnRzCiogYGVgIC0gVGhlIHJ1bnRpbWUgZW52aXJvbm1lbnQuCiogYHRvYCAtIFRoZSBhZGRyZXNzIHdoZXJlIHRoZSBtaW50ZWQgTFAgdG9rZW5zIHdpbGwgYmUgc2VudC4KCiMgUmV0dXJucwpUaGUgYW1vdW50IG9mIG1pbnRlZCBMUCB0b2tlbnMuClBvc3NpYmxlIGVycm9yczoKLSBgU29yb3N3YXBQYWlyRXJyb3I6Ok5vdEluaXRpYWxpemVkYDogVGhlIFNvcm9zd2FwIHBhaXIgaGFzIG5vdCBiZWVuIGluaXRpYWxpemVkLgotIGBTb3Jvc3dhcFBhaXJFcnJvcjo6RGVwb3NpdEluc3VmZmljaWVudEFtb3VudFRva2VuMGA6IEluc3VmZmljaWVudCBhbW91bnQgb2YgdG9rZW4gMCBzZW50LgotIGBTb3Jvc3dhcFBhaXJFcnJvcjo6RGVwb3NpdEluc3VmZmljaWVudEFtb3VudFRva2VuMWA6IEluc3VmZmljaWVudCBhbW91bnQgb2YgdG9rZW4gMSBzZW50LgotIGBTb3Jvc3dhcFBhaXJFcnJvcjo6RGVwb3NpdEluc3VmZmljaWVudEZpcnN0TGlxdWlkaXR5YDogSW5zdWZmaWNpZW50IGZpcnN0IGxpcXVpZGl0eSBtaW50ZWQuCi0gYFNvcm9zd2FwUGFpckVycm9yOjpEZXBvc2l0SW5zdWZmaWNpZW50TGlxdWlkaXR5TWludGVkYDogSW5zdWZmaWNpZW50IGxpcXVpZGl0eSBtaW50ZWQuCi0gYFNvcm9zd2FwUGFpckVycm9yOjpVcGRhdGVPdmVyZmxvd2A6IE92ZXJmbG93IG9jY3VycmVkIGR1cmluZyB1cGRhdGUuAAAAAAdkZXBvc2l0AAAAAAEAAAAAAAAAAnRvAAAAAAATAAAAAQAAA+kAAAALAAAH0AAAABFTb3Jvc3dhcFBhaXJFcnJvcgAAAA==",
        "AAAAAAAAADVSZXR1cm5zIHRoZSBhZGRyZXNzIG9mIHRoZSBTb3Jvc3dhcCBmYWN0b3J5IGNvbnRyYWN0LgAAAAAAAAdmYWN0b3J5AAAAAAAAAAABAAAAEw==",
        "AAAAAAAAADxSZXR1cm5zIHRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbiBpbiB0aGUgU29yb3N3YXAgcGFpci4AAAAHdG9rZW5fMAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAD1SZXR1cm5zIHRoZSBhZGRyZXNzIG9mIHRoZSBzZWNvbmQgdG9rZW4gaW4gdGhlIFNvcm9zd2FwIHBhaXIuAAAAAAAAB3Rva2VuXzEAAAAAAAAAAAEAAAAT",
        "AAAAAAAAATdXaXRoZHJhd3MgbGlxdWlkaXR5IGZyb20gdGhlIFNvcm9zd2FwIHBhaXIsIGJ1cm5pbmcgTFAgdG9rZW5zIGFuZCByZXR1cm5pbmcgdGhlIGNvcnJlc3BvbmRpbmcgdG9rZW5zIHRvIHRoZSB1c2VyLgoKIyBBcmd1bWVudHMKKiBgZWAgLSBUaGUgcnVudGltZSBlbnZpcm9ubWVudC4KKiBgdG9gIC0gVGhlIGFkZHJlc3Mgd2hlcmUgdGhlIHdpdGhkcmF3biB0b2tlbnMgd2lsbCBiZSBzZW50LgoKIyBSZXR1cm5zCkEgdHVwbGUgY29udGFpbmluZyB0aGUgYW1vdW50cyBvZiB0b2tlbiAwIGFuZCB0b2tlbiAxIHdpdGhkcmF3biBmcm9tIHRoZSBwYWlyLgAAAAAId2l0aGRyYXcAAAABAAAAAAAAAAJ0bwAAAAAAEwAAAAEAAAPpAAAD7QAAAAIAAAALAAAACwAAB9AAAAARU29yb3N3YXBQYWlyRXJyb3IAAAA=",
        "AAAAAAAAATlJbml0aWFsaXplcyBhIG5ldyBTb3Jvc3dhcCBwYWlyIGJ5IHNldHRpbmcgdG9rZW4gYWRkcmVzc2VzLCBmYWN0b3J5LCBhbmQgaW5pdGlhbCByZXNlcnZlcy4KCiMgQXJndW1lbnRzCiogYGVgIC0gVGhlIHJ1bnRpbWUgZW52aXJvbm1lbnQuCiogYGZhY3RvcnlgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIFNvcm9zd2FwIGZhY3RvcnkgY29udHJhY3QuCiogYHRva2VuXzBgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGZpcnN0IHRva2VuIGluIHRoZSBwYWlyLgoqIGB0b2tlbl8xYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBzZWNvbmQgdG9rZW4gaW4gdGhlIHBhaXIuAAAAAAAACmluaXRpYWxpemUAAAAAAAMAAAAAAAAAB2ZhY3RvcnkAAAAAEwAAAAAAAAAHdG9rZW5fMAAAAAATAAAAAAAAAAd0b2tlbl8xAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAABFTb3Jvc3dhcFBhaXJFcnJvcgAAAA==",
        "AAAAAAAAAKtSZXR1cm5zIHRoZSBjdXJyZW50IHJlc2VydmVzIGFuZCB0aGUgbGFzdCBibG9jayB0aW1lc3RhbXAuCgojIEFyZ3VtZW50cwoqIGBlYCAtIFRoZSBydW50aW1lIGVudmlyb25tZW50LgoKIyBSZXR1cm5zCkEgdHVwbGUgY29udGFpbmluZyB0aGUgcmVzZXJ2ZXMgb2YgdG9rZW4gMCBhbmQgdG9rZW4gMS4AAAAADGdldF9yZXNlcnZlcwAAAAAAAAABAAAD7QAAAAIAAAALAAAACw==",
        "AAAABAAAAAAAAAAAAAAAEVNvcm9zd2FwUGFpckVycm9yAAAAAAAAEgAAACFTb3Jvc3dhcFBhaXI6IGFscmVhZHkgaW5pdGlhbGl6ZWQAAAAAAAAcSW5pdGlhbGl6ZUFscmVhZHlJbml0aWFsaXplZAAAAGUAAAAhU29yb3N3YXBQYWlyOiBub3QgeWV0IGluaXRpYWxpemVkAAAAAAAADk5vdEluaXRpYWxpemVkAAAAAABmAAAAQlNvcm9zd2FwUGFpcjogdG9rZW5fMCBtdXN0IGJlIGxlc3MgdGhhbiB0b2tlbl8xIHdoaWxlIGluaXRpYWxpemluZwAAAAAAG0luaXRpYWxpemVUb2tlbk9yZGVySW52YWxpZAAAAABnAAAARVNvcm9zd2FwUGFpcjogaW5zdWZmaWNpZW50IGFtb3VudCBvZiB0b2tlbiAwIHNlbnQgd2hpbGUgZG9pbmcgZGVwb3NpdAAAAAAAAB9EZXBvc2l0SW5zdWZmaWNpZW50QW1vdW50VG9rZW4wAAAAAGgAAABFU29yb3N3YXBQYWlyOiBpbnN1ZmZpY2llbnQgYW1vdW50IG9mIHRva2VuIDEgc2VudCB3aGlsZSBkb2luZyBkZXBvc2l0AAAAAAAAH0RlcG9zaXRJbnN1ZmZpY2llbnRBbW91bnRUb2tlbjEAAAAAaQAAAEVTb3Jvc3dhcFBhaXI6IGluc3VmZmljaWVudCBmaXJzdCBsaXF1aWRpdHkgbWludGVkIHdoaWxlIGRvaW5nIGRlcG9zaXQAAAAAAAAhRGVwb3NpdEluc3VmZmljaWVudEZpcnN0TGlxdWlkaXR5AAAAAAAAagAAAD9Tb3Jvc3dhcFBhaXI6IGluc3VmZmljaWVudCBsaXF1aWRpdHkgbWludGVkIHdoaWxlIGRvaW5nIGRlcG9zaXQAAAAAIkRlcG9zaXRJbnN1ZmZpY2llbnRMaXF1aWRpdHlNaW50ZWQAAAAAAGsAAABDU29yb3N3YXBQYWlyOiBpbnN1ZmZpY2llbnQgb3V0cHV0IGFtb3VudCB3aGlsZSBkb2luZyBkZXBvc0RlcG9zaXRpdAAAAAAcU3dhcEluc3VmZmljaWVudE91dHB1dEFtb3VudAAAAGwAAABDU29yb3N3YXBQYWlyOiBuZWdhdGl2ZXMgYW1vdW50cyBvdXQgZG9udCBzdXBwb3J0ZWQgd2hpbGUgZG9pbmcgc3dhcAAAAAAcU3dhcE5lZ2F0aXZlc091dE5vdFN1cHBvcnRlZAAAAG0AAAAzU29yb3N3YXBQYWlyOiBpbnN1ZmZpY2llbnQgbGlxdWlkaXR5IHRvIGRvIHRoZSBzd2FwAAAAABlTd2FwSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAAAbgAAACdTb3Jvc3dhcFBhaXI6IGludmFsaWQgdG8gdG8gZG8gdGhlIHN3YXAAAAAADVN3YXBJbnZhbGlkVG8AAAAAAABvAAAAOFNvcm9zd2FwUGFpcjogaW5zdWZmaWNpZW50IGlucHV0IGFtb3VudCB3aGlsZSBkb2luZyBzd2FwAAAAG1N3YXBJbnN1ZmZpY2llbnRJbnB1dEFtb3VudAAAAABwAAAAQlNvcm9zd2FwUGFpcjogbmVnYXRpdmVzIGFtb3VudHMgaW4gZG9udCBzdXBwb3J0ZWQgd2hpbGUgZG9pbmcgc3dhcAAAAAAAG1N3YXBOZWdhdGl2ZXNJbk5vdFN1cHBvcnRlZAAAAABxAAAANFNvcm9zd2FwUGFpcjogSyBjb25zdGFudCBpcyBub3QgbWV0IHdoaWxlIGRvaW5nIHN3YXAAAAATU3dhcEtDb25zdGFudE5vdE1ldAAAAAByAAAARFNvcm9zd2FwUGFpcjogbGlxdWlkaXR5IHdhcyBub3QgaW5pdGlhbGl6ZWQgeWV0IHdoaWxlIGRvaW5nIHdpdGhkcmF3AAAAH1dpdGhkcmF3TGlxdWlkaXR5Tm90SW5pdGlhbGl6ZWQAAAAAcwAAADtTb3Jvc3dhcFBhaXI6IGluc3VmZmljaWVudCBzZW50IHNoYXJlcyB3aGlsZSBkb2luZyB3aXRoZHJhdwAAAAAeV2l0aGRyYXdJbnN1ZmZpY2llbnRTZW50U2hhcmVzAAAAAAB0AAAAQFNvcm9zd2FwUGFpcjogaW5zdWZmaWNpZW50IGxpcXVpZGl0eSBidXJuZWQgd2hpbGUgZG9pbmcgd2l0aGRyYXcAAAAjV2l0aGRyYXdJbnN1ZmZpY2llbnRMaXF1aWRpdHlCdXJuZWQAAAAAdQAAACVTb3Jvc3dhcFBhaXI6IE9WRVJGTE9XIHdoaWxlIHVwZGF0aW5nAAAAAAAADlVwZGF0ZU92ZXJmbG93AAAAAAB2",
        "AAAAAQAAAAAAAAAAAAAACVNraW1FdmVudAAAAAAAAAIAAAAAAAAACXNraW1tZWRfMAAAAAAAAAsAAAAAAAAACXNraW1tZWRfMQAAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAACVN3YXBFdmVudAAAAAAAAAUAAAAAAAAAC2Ftb3VudF8wX2luAAAAAAsAAAAAAAAADGFtb3VudF8wX291dAAAAAsAAAAAAAAAC2Ftb3VudF8xX2luAAAAAAsAAAAAAAAADGFtb3VudF8xX291dAAAAAsAAAAAAAAAAnRvAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAACVN5bmNFdmVudAAAAAAAAAIAAAAAAAAADW5ld19yZXNlcnZlXzAAAAAAAAALAAAAAAAAAA1uZXdfcmVzZXJ2ZV8xAAAAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAADERlcG9zaXRFdmVudAAAAAYAAAAAAAAACGFtb3VudF8wAAAACwAAAAAAAAAIYW1vdW50XzEAAAALAAAAAAAAAAlsaXF1aWRpdHkAAAAAAAALAAAAAAAAAA1uZXdfcmVzZXJ2ZV8wAAAAAAAACwAAAAAAAAANbmV3X3Jlc2VydmVfMQAAAAAAAAsAAAAAAAAAAnRvAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAADVdpdGhkcmF3RXZlbnQAAAAAAAAGAAAAAAAAAAhhbW91bnRfMAAAAAsAAAAAAAAACGFtb3VudF8xAAAACwAAAAAAAAAJbGlxdWlkaXR5AAAAAAAACwAAAAAAAAANbmV3X3Jlc2VydmVfMAAAAAAAAAsAAAAAAAAADW5ld19yZXNlcnZlXzEAAAAAAAALAAAAAAAAAAJ0bwAAAAAAEw==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAwAAAAEAAAAAAAAACUFsbG93YW5jZQAAAAAAAAEAAAfQAAAAEEFsbG93YW5jZURhdGFLZXkAAAABAAAAAAAAAAdCYWxhbmNlAAAAAAEAAAATAAAAAAAAAAAAAAALVG90YWxTdXBwbHkA",
        "AAAAAQAAAAAAAAAAAAAADkFsbG93YW5jZVZhbHVlAAAAAAACAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAEWV4cGlyYXRpb25fbGVkZ2VyAAAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAEEFsbG93YW5jZURhdGFLZXkAAAACAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAHc3BlbmRlcgAAAAAT",
        "AAAAAAAAAAAAAAAEYnVybgAAAAIAAAAAAAAABGZyb20AAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAA",
        "AAAAAAAAAAAAAAAEbmFtZQAAAAAAAAABAAAAEA==",
        "AAAAAAAAAAAAAAAGc3ltYm9sAAAAAAAAAAAAAQAAABA=",
        "AAAAAAAAAAAAAAAHYXBwcm92ZQAAAAAEAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAHc3BlbmRlcgAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAEWV4cGlyYXRpb25fbGVkZ2VyAAAAAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAHYmFsYW5jZQAAAAABAAAAAAAAAAJpZAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAIZGVjaW1hbHMAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAIdHJhbnNmZXIAAAADAAAAAAAAAARmcm9tAAAAEwAAAAAAAAACdG8AAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAA=",
        "AAAAAAAAAAAAAAAJYWxsb3dhbmNlAAAAAAAAAgAAAAAAAAAEZnJvbQAAABMAAAAAAAAAB3NwZW5kZXIAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAJYnVybl9mcm9tAAAAAAAAAwAAAAAAAAAHc3BlbmRlcgAAAAATAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAA==",
        "AAAAAAAAAAAAAAAMdG90YWxfc3VwcGx5AAAAAAAAAAEAAAAL",
        "AAAAAAAAAAAAAAANdHJhbnNmZXJfZnJvbQAAAAAAAAQAAAAAAAAAB3NwZW5kZXIAAAAAEwAAAAAAAAAEZnJvbQAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAA",
        "AAAAAQAAAAAAAAAAAAAADVRva2VuTWV0YWRhdGEAAAAAAAADAAAAAAAAAAdkZWNpbWFsAAAAAAQAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZzeW1ib2wAAAAAABA=",
        "AAAABAAAAAAAAAAAAAAADEZhY3RvcnlFcnJvcgAAAAYAAAAkU29yb3N3YXBGYWN0b3J5OiBub3QgeWV0IGluaXRpYWxpemVkAAAADk5vdEluaXRpYWxpemVkAAAAAADJAAAAPVNvcm9zd2FwRmFjdG9yeTogdG9rZW5fYSBhbmQgdG9rZW5fYiBoYXZlIGlkZW50aWNhbCBhZGRyZXNzZXMAAAAAAAAZQ3JlYXRlUGFpcklkZW50aWNhbFRva2VucwAAAAAAAMoAAABAU29yb3N3YXBGYWN0b3J5OiBwYWlyIGFscmVhZHkgZXhpc3RzIGJldHdlZW4gdG9rZW5fYSBhbmQgdG9rZW5fYgAAABdDcmVhdGVQYWlyQWxyZWFkeUV4aXN0cwAAAADLAAAAJFNvcm9zd2FwRmFjdG9yeTogYWxyZWFkeSBpbml0aWFsaXplZAAAABxJbml0aWFsaXplQWxyZWFkeUluaXRpYWxpemVkAAAAzAAAACRTb3Jvc3dhcEZhY3Rvcnk6IHBhaXIgZG9lcyBub3QgZXhpc3QAAAAQUGFpckRvZXNOb3RFeGlzdAAAAM0AAAAlU29yb3N3YXBGYWN0b3J5OiBpbmRleCBkb2VzIG5vdCBleGlzdAAAAAAAABFJbmRleERvZXNOb3RFeGlzdAAAAAAAAM4=" ]),
      options
    )
  }
  public readonly fromJSON = {
    skim: this.txFromJSON<null>,
        swap: this.txFromJSON<Result<void>>,
        sync: this.txFromJSON<null>,
        k_last: this.txFromJSON<i128>,
        deposit: this.txFromJSON<Result<i128>>,
        factory: this.txFromJSON<string>,
        token_0: this.txFromJSON<string>,
        token_1: this.txFromJSON<string>,
        withdraw: this.txFromJSON<Result<readonly [i128, i128]>>,
        initialize: this.txFromJSON<Result<void>>,
        get_reserves: this.txFromJSON<readonly [i128, i128]>,
        burn: this.txFromJSON<null>,
        name: this.txFromJSON<string>,
        symbol: this.txFromJSON<string>,
        approve: this.txFromJSON<null>,
        balance: this.txFromJSON<i128>,
        decimals: this.txFromJSON<u32>,
        transfer: this.txFromJSON<null>,
        allowance: this.txFromJSON<i128>,
        burn_from: this.txFromJSON<null>,
        total_supply: this.txFromJSON<i128>,
        transfer_from: this.txFromJSON<null>
  }
}