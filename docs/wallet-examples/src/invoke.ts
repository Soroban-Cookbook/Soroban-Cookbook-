/**
 * Wallet-agnostic contract invocation: build → simulate/prepare → sign with
 * any WalletAdapter → submit → poll. Swap the adapter, nothing else changes.
 */
import {
  BASE_FEE,
  Contract,
  Networks,
  TransactionBuilder,
  rpc,
  xdr,
} from "@stellar/stellar-sdk";
import { NetworkName, WalletAdapter } from "./types";

export const NETWORKS: Record<NetworkName, { passphrase: string; rpcUrl: string }> = {
  TESTNET: {
    passphrase: Networks.TESTNET,
    rpcUrl: "https://soroban-testnet.stellar.org",
  },
  PUBLIC: {
    passphrase: Networks.PUBLIC,
    // SDF does not run a public mainnet RPC; use your provider's URL.
    rpcUrl: "https://<your-rpc-provider>/",
  },
};

export async function invokeContract(
  wallet: WalletAdapter,
  address: string,
  network: NetworkName,
  contractId: string,
  method: string,
  args: xdr.ScVal[] = [],
): Promise<rpc.Api.GetSuccessfulTransactionResponse> {
  const { passphrase, rpcUrl } = NETWORKS[network];
  const server = new rpc.Server(rpcUrl);

  const source = await server.getAccount(address);
  const tx = new TransactionBuilder(source, { fee: BASE_FEE, networkPassphrase: passphrase })
    .addOperation(new Contract(contractId).call(method, ...args))
    .setTimeout(60)
    .build();

  // Simulation adds the Soroban footprint, resource fees, and auth entries.
  const prepared = await server.prepareTransaction(tx);

  const signedXdr = await wallet.signTransaction(prepared.toXDR(), {
    address,
    networkPassphrase: passphrase,
  });
  const signed = TransactionBuilder.fromXDR(signedXdr, passphrase);

  const sent = await server.sendTransaction(signed);
  if (sent.status === "ERROR") {
    throw new Error(`Submission failed: ${sent.errorResult?.toXDR("base64")}`);
  }

  for (let i = 0; i < 30; i++) {
    const res = await server.getTransaction(sent.hash);
    if (res.status === rpc.Api.GetTransactionStatus.SUCCESS) return res;
    if (res.status === rpc.Api.GetTransactionStatus.FAILED) {
      throw new Error(`Transaction ${sent.hash} failed on-chain`);
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`Timed out waiting for ${sent.hash}`);
}
