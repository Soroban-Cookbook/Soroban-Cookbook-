/**
 * One integration for many wallets: Stellar Wallets Kit ships modules for
 * Freighter, xBull, Albedo, Rabet, LOBSTR, Hana, Hot Wallet, Ledger, Trezor
 * and WalletConnect, plus a ready-made selection modal.
 *
 * Package: @creit.tech/stellar-wallets-kit
 */
import {
  allowAllModules,
  FREIGHTER_ID,
  ISupportedWallet,
  StellarWalletsKit,
  WalletNetwork,
} from "@creit.tech/stellar-wallets-kit";
import { SignOptions, WalletAdapter, WalletRejectedError } from "./types";

export function createWalletsKitAdapter(network: WalletNetwork): WalletAdapter {
  const kit = new StellarWalletsKit({
    network,
    selectedWalletId: FREIGHTER_ID,
    modules: allowAllModules(),
  });

  return {
    id: "wallets-kit",
    name: "Stellar Wallets Kit",

    async isAvailable() {
      return true; // The modal lists only wallets the user actually has.
    },

    connect() {
      return new Promise<string>((resolve, reject) => {
        kit
          .openModal({
            onWalletSelected: async (option: ISupportedWallet) => {
              try {
                kit.setWallet(option.id);
                const { address } = await kit.getAddress();
                resolve(address);
              } catch {
                reject(new WalletRejectedError(option.name));
              }
            },
            onClosed: () => reject(new WalletRejectedError("Stellar Wallets Kit")),
          })
          .catch(reject);
      });
    },

    async signTransaction(xdr: string, opts: SignOptions) {
      try {
        const { signedTxXdr } = await kit.signTransaction(xdr, {
          address: opts.address,
          networkPassphrase: opts.networkPassphrase,
        });
        return signedTxXdr;
      } catch {
        throw new WalletRejectedError("Stellar Wallets Kit");
      }
    },

    async disconnect() {
      await kit.disconnect();
    },
  };
}
