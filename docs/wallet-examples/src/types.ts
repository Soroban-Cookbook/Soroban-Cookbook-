/**
 * Common shape every wallet example implements, so the dApp code (connect →
 * build → sign → submit) is identical regardless of which wallet the user picks.
 */
export type NetworkName = "TESTNET" | "PUBLIC";

export interface WalletAdapter {
  /** Stable id used in UI and persisted preferences. */
  readonly id: string;
  /** Human-readable name. */
  readonly name: string;
  /** True when the wallet is reachable (extension injected, device plugged in, …). */
  isAvailable(): Promise<boolean>;
  /** Ask the user to connect and return their G... address. */
  connect(): Promise<string>;
  /** Sign a transaction envelope (base64 XDR) and return the signed XDR. */
  signTransaction(xdr: string, opts: SignOptions): Promise<string>;
  /** Forget any session state held by the adapter. */
  disconnect(): Promise<void>;
}

export interface SignOptions {
  address: string;
  networkPassphrase: string;
}

/** Thrown when the user dismisses or rejects a wallet prompt. */
export class WalletRejectedError extends Error {
  constructor(walletName: string) {
    super(`${walletName}: request rejected by user`);
    this.name = "WalletRejectedError";
  }
}

/** Thrown when the wallet is not installed / not connected. */
export class WalletUnavailableError extends Error {
  constructor(walletName: string) {
    super(`${walletName}: wallet not available`);
    this.name = "WalletUnavailableError";
  }
}
