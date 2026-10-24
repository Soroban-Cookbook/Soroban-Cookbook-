/**
 * Hana Wallet (browser extension + mobile). Hana injects
 * `window.hanaWallet.stellar` into every page.
 *
 * Docs: https://hana.network
 */
import {
  SignOptions,
  WalletAdapter,
  WalletRejectedError,
  WalletUnavailableError,
} from "./types";

interface HanaStellarProvider {
  getPublicKey(): Promise<string>;
  signTransaction(params: {
    xdr: string;
    accountToSign?: string;
    networkPassphrase?: string;
  }): Promise<string>;
}

declare global {
  interface Window {
    hanaWallet?: { stellar?: HanaStellarProvider };
  }
}

function provider(): HanaStellarProvider {
  const p = typeof window !== "undefined" ? window.hanaWallet?.stellar : undefined;
  if (!p) throw new WalletUnavailableError("Hana");
  return p;
}

export const hanaAdapter: WalletAdapter = {
  id: "hana",
  name: "Hana",

  async isAvailable() {
    return typeof window !== "undefined" && !!window.hanaWallet?.stellar;
  },

  async connect() {
    try {
      return await provider().getPublicKey();
    } catch (err) {
      if (err instanceof WalletUnavailableError) throw err;
      throw new WalletRejectedError("Hana");
    }
  },

  async signTransaction(xdr: string, opts: SignOptions) {
    try {
      return await provider().signTransaction({
        xdr,
        accountToSign: opts.address,
        networkPassphrase: opts.networkPassphrase,
      });
    } catch (err) {
      if (err instanceof WalletUnavailableError) throw err;
      throw new WalletRejectedError("Hana");
    }
  },

  async disconnect() {
    // Hana keeps no dApp session; clearing app state is enough.
  },
};
