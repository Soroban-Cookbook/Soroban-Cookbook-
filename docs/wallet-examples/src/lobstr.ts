/**
 * LOBSTR via the LOBSTR Signer Extension (desktop browser). For LOBSTR mobile,
 * connect over WalletConnect instead — see `wallets-kit.ts`, which bundles the
 * WalletConnect module.
 *
 * Package: @lobstrco/signer-extension-api
 */
import {
  getPublicKey,
  isConnected,
  signTransaction,
} from "@lobstrco/signer-extension-api";
import {
  SignOptions,
  WalletAdapter,
  WalletRejectedError,
  WalletUnavailableError,
} from "./types";

export const lobstrAdapter: WalletAdapter = {
  id: "lobstr",
  name: "LOBSTR",

  async isAvailable() {
    try {
      return await isConnected();
    } catch {
      return false;
    }
  },

  async connect() {
    if (!(await this.isAvailable())) throw new WalletUnavailableError("LOBSTR");
    const address = await getPublicKey();
    if (!address) throw new WalletRejectedError("LOBSTR");
    return address;
  },

  // The signer extension signs for the account the user picked in `connect`
  // and does not take a network argument: it signs the envelope as given, so
  // the network passphrase is already baked into the transaction hash.
  async signTransaction(xdr: string, _opts: SignOptions) {
    if (!(await this.isAvailable())) throw new WalletUnavailableError("LOBSTR");
    const signed = await signTransaction(xdr);
    if (!signed) throw new WalletRejectedError("LOBSTR");
    return signed;
  },

  async disconnect() {
    // No session to tear down for the extension.
  },
};
