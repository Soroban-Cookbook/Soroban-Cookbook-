/**
 * Ledger hardware wallet over WebUSB (Chrome/Edge/Brave). The device must have
 * the Stellar app open. Soroban invocations need Stellar app v5 or newer.
 *
 * Packages: @ledgerhq/hw-transport-webusb, @ledgerhq/hw-app-str
 */
import Str from "@ledgerhq/hw-app-str";
import TransportWebUSB from "@ledgerhq/hw-transport-webusb";
import type Transport from "@ledgerhq/hw-transport";
import { StrKey, TransactionBuilder } from "@stellar/stellar-sdk";
import {
  SignOptions,
  WalletAdapter,
  WalletRejectedError,
  WalletUnavailableError,
} from "./types";

/** First Stellar account (SEP-0005). Change the last index for other accounts. */
export const DEFAULT_LEDGER_PATH = "44'/148'/0'";

// Ledger status word returned when the user presses "Reject" on the device.
const SW_DENIED = 0x6985;

let transport: Transport | null = null;

async function app(): Promise<Str> {
  if (!transport) {
    try {
      transport = await TransportWebUSB.create();
    } catch {
      throw new WalletUnavailableError("Ledger");
    }
  }
  return new Str(transport);
}

function isDenied(err: unknown): boolean {
  return (err as { statusCode?: number })?.statusCode === SW_DENIED;
}

export function createLedgerAdapter(path = DEFAULT_LEDGER_PATH): WalletAdapter {
  return {
    id: "ledger",
    name: "Ledger",

    async isAvailable() {
      return TransportWebUSB.isSupported();
    },

    async connect() {
      const str = await app();
      const { rawPublicKey } = await str.getPublicKey(path);
      return StrKey.encodeEd25519PublicKey(rawPublicKey);
    },

    async signTransaction(xdr: string, opts: SignOptions) {
      const str = await app();
      const tx = TransactionBuilder.fromXDR(xdr, opts.networkPassphrase);
      try {
        // The device parses and displays the envelope (signature base), then
        // returns a raw ed25519 signature over its hash.
        const { signature } = await str.signTransaction(path, tx.signatureBase());
        tx.addSignature(opts.address, signature.toString("base64"));
        return tx.toXDR();
      } catch (err) {
        if (isDenied(err)) throw new WalletRejectedError("Ledger");
        throw err;
      }
    },

    async disconnect() {
      await transport?.close();
      transport = null;
    },
  };
}

export const ledgerAdapter = createLedgerAdapter();
