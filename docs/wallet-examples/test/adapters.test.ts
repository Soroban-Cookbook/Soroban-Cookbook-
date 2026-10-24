/**
 * Unit tests for the wallet adapters. Real wallets are replaced with fakes:
 * injected providers are stubbed on `window`, npm SDKs are mocked with vi.mock.
 * Run: npm test   (inside docs/wallet-examples)
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  Account,
  Keypair,
  Networks,
  Operation,
  TransactionBuilder,
  hash,
} from "@stellar/stellar-sdk";
import { WalletRejectedError, WalletUnavailableError } from "../src/types";

const user = Keypair.random();
const PASSPHRASE = Networks.TESTNET;

function unsignedXdr(): string {
  return new TransactionBuilder(new Account(user.publicKey(), "1"), {
    fee: "100",
    networkPassphrase: PASSPHRASE,
  })
    .addOperation(Operation.bumpSequence({ bumpTo: "2" }))
    .setTimeout(60)
    .build()
    .toXDR();
}

// ---------------------------------------------------------------------------
// Hana (injected provider)
// ---------------------------------------------------------------------------

describe("hanaAdapter", () => {
  afterEach(() => {
    delete window.hanaWallet;
  });

  it("reports unavailable when the extension is missing", async () => {
    const { hanaAdapter } = await import("../src/hana");
    expect(await hanaAdapter.isAvailable()).toBe(false);
    await expect(hanaAdapter.connect()).rejects.toBeInstanceOf(WalletUnavailableError);
  });

  it("connects and signs through window.hanaWallet.stellar", async () => {
    const signTransaction = vi.fn().mockResolvedValue("SIGNED_XDR");
    window.hanaWallet = {
      stellar: {
        getPublicKey: vi.fn().mockResolvedValue(user.publicKey()),
        signTransaction,
      },
    };
    const { hanaAdapter } = await import("../src/hana");

    expect(await hanaAdapter.isAvailable()).toBe(true);
    expect(await hanaAdapter.connect()).toBe(user.publicKey());

    const xdr = unsignedXdr();
    const signed = await hanaAdapter.signTransaction(xdr, {
      address: user.publicKey(),
      networkPassphrase: PASSPHRASE,
    });
    expect(signed).toBe("SIGNED_XDR");
    expect(signTransaction).toHaveBeenCalledWith({
      xdr,
      accountToSign: user.publicKey(),
      networkPassphrase: PASSPHRASE,
    });
  });

  it("maps a rejected prompt to WalletRejectedError", async () => {
    window.hanaWallet = {
      stellar: {
        getPublicKey: vi.fn().mockRejectedValue(new Error("User declined")),
        signTransaction: vi.fn(),
      },
    };
    const { hanaAdapter } = await import("../src/hana");
    await expect(hanaAdapter.connect()).rejects.toBeInstanceOf(WalletRejectedError);
  });
});

// ---------------------------------------------------------------------------
// LOBSTR (signer extension SDK)
// ---------------------------------------------------------------------------

const lobstrApi = vi.hoisted(() => ({
  isConnected: vi.fn(),
  getPublicKey: vi.fn(),
  signTransaction: vi.fn(),
}));
vi.mock("@lobstrco/signer-extension-api", () => lobstrApi);

describe("lobstrAdapter", () => {
  beforeEach(() => vi.clearAllMocks());

  it("throws WalletUnavailableError when the extension is not connected", async () => {
    lobstrApi.isConnected.mockResolvedValue(false);
    const { lobstrAdapter } = await import("../src/lobstr");
    await expect(lobstrAdapter.connect()).rejects.toBeInstanceOf(WalletUnavailableError);
  });

  it("connects and signs", async () => {
    lobstrApi.isConnected.mockResolvedValue(true);
    lobstrApi.getPublicKey.mockResolvedValue(user.publicKey());
    lobstrApi.signTransaction.mockResolvedValue("SIGNED_XDR");
    const { lobstrAdapter } = await import("../src/lobstr");

    expect(await lobstrAdapter.connect()).toBe(user.publicKey());
    expect(
      await lobstrAdapter.signTransaction(unsignedXdr(), {
        address: user.publicKey(),
        networkPassphrase: PASSPHRASE,
      }),
    ).toBe("SIGNED_XDR");
  });

  it("treats an empty signature as a rejection", async () => {
    lobstrApi.isConnected.mockResolvedValue(true);
    lobstrApi.signTransaction.mockResolvedValue("");
    const { lobstrAdapter } = await import("../src/lobstr");
    await expect(
      lobstrAdapter.signTransaction(unsignedXdr(), {
        address: user.publicKey(),
        networkPassphrase: PASSPHRASE,
      }),
    ).rejects.toBeInstanceOf(WalletRejectedError);
  });
});

// ---------------------------------------------------------------------------
// Ledger (WebUSB transport + Stellar app)
// ---------------------------------------------------------------------------

const ledgerMocks = vi.hoisted(() => ({
  getPublicKey: vi.fn(),
  signTransaction: vi.fn(),
  close: vi.fn(),
}));
vi.mock("@ledgerhq/hw-transport-webusb", () => ({
  default: {
    isSupported: vi.fn().mockResolvedValue(true),
    create: vi.fn().mockResolvedValue({ close: ledgerMocks.close }),
  },
}));
vi.mock("@ledgerhq/hw-app-str", () => ({
  default: vi.fn().mockImplementation(() => ({
    getPublicKey: ledgerMocks.getPublicKey,
    signTransaction: ledgerMocks.signTransaction,
  })),
}));

describe("ledgerAdapter", () => {
  beforeEach(() => vi.clearAllMocks());

  it("derives the G... address from the device public key", async () => {
    ledgerMocks.getPublicKey.mockResolvedValue({ rawPublicKey: user.rawPublicKey() });
    const { ledgerAdapter, DEFAULT_LEDGER_PATH } = await import("../src/ledger");

    expect(await ledgerAdapter.connect()).toBe(user.publicKey());
    expect(ledgerMocks.getPublicKey).toHaveBeenCalledWith(DEFAULT_LEDGER_PATH);
  });

  it("attaches a valid device signature to the envelope", async () => {
    // The fake device signs with the test keypair, so the signature verifies.
    ledgerMocks.signTransaction.mockImplementation(async (_path: string, base: Buffer) => ({
      signature: user.sign(hash(base)),
    }));
    const { ledgerAdapter } = await import("../src/ledger");

    const signedXdr = await ledgerAdapter.signTransaction(unsignedXdr(), {
      address: user.publicKey(),
      networkPassphrase: PASSPHRASE,
    });
    const tx = TransactionBuilder.fromXDR(signedXdr, PASSPHRASE);
    expect(tx.signatures).toHaveLength(1);
    expect(user.verify(tx.hash(), tx.signatures[0].signature())).toBe(true);
  });

  it("maps the device 'Reject' status word to WalletRejectedError", async () => {
    ledgerMocks.signTransaction.mockRejectedValue({ statusCode: 0x6985 });
    const { ledgerAdapter } = await import("../src/ledger");
    await expect(
      ledgerAdapter.signTransaction(unsignedXdr(), {
        address: user.publicKey(),
        networkPassphrase: PASSPHRASE,
      }),
    ).rejects.toBeInstanceOf(WalletRejectedError);
  });
});
