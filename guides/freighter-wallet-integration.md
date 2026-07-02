# Freighter Wallet Integration Guide

This guide covers how to integrate the Freighter Wallet with your Soroban dApp. It includes practical examples demonstrating wallet connection, account retrieval, network detection, transaction construction, transaction signing, and transaction submission, along with a setup and troubleshooting guide.

## 1. Setup Guide

To get started, you'll need to install the `@stellar/freighter-api` package in your web app:

```bash
npm install @stellar/freighter-api
# or
yarn add @stellar/freighter-api
```

## 2. Connection & Network Detection

Before requesting the user's account, you should check if Freighter is installed and on the correct network.

```javascript
import { isConnected, getNetwork, getNetworkDetails } from "@stellar/freighter-api";

async function checkFreighterStatus() {
  if (await isConnected()) {
    console.log("Freighter is installed and connected!");
    const network = await getNetwork();
    console.log(`Connected to network: ${network}`);
    const details = await getNetworkDetails();
    console.log(`Network details: ${JSON.stringify(details)}`);
  } else {
    console.log("Freighter is not installed or not connected.");
  }
}
```

## 3. Account Retrieval

To retrieve the user's public key from Freighter:

```javascript
import { requestAccess, setAllowed } from "@stellar/freighter-api";

async function retrieveAccount() {
  try {
    const publicKey = await requestAccess();
    console.log(`User's public key: ${publicKey}`);
    // Optional: Keep the site authorized for future interactions
    await setAllowed();
    return publicKey;
  } catch (error) {
    console.error("User denied access or another error occurred:", error);
  }
}
```

## 4. Transaction Construction

You can construct your transaction using the `stellar-sdk` library. Here's an example of how you might invoke a Soroban contract.

```javascript
import { SorobanRpc, TransactionBuilder, xdr, Networks } from "@stellar/stellar-sdk";

async function buildTransaction(publicKey, contractId, method, args) {
  const server = new SorobanRpc.Server("https://soroban-testnet.stellar.org");
  const account = await server.getAccount(publicKey);

  const tx = new TransactionBuilder(account, { fee: "100", networkPassphrase: Networks.TESTNET })
    .addOperation(xdr.Operation.invokeHostFunction({
      hostFunction: xdr.HostFunction.hostFunctionTypeInvokeContract(
        new xdr.InvokeContractArgs({
          contractAddress: new xdr.ScAddress.scAddressTypeContract(Buffer.from(contractId, 'hex')),
          functionName: method,
          args: args
        })
      ),
      auth: []
    }))
    .setTimeout(30)
    .build();

  // Simulate transaction to get footprint...
  const simulatedTx = await server.simulateTransaction(tx);
  // Assemble tx with footprint...
  
  return tx.toXDR();
}
```

## 5. Transaction Signing

After building your transaction XDR, pass it to Freighter to be signed.

```javascript
import { signTransaction } from "@stellar/freighter-api";

async function signWithFreighter(transactionXdr, networkPassphrase) {
  try {
    const signedXdr = await signTransaction(transactionXdr, {
      networkPassphrase: networkPassphrase,
    });
    console.log("Transaction successfully signed!");
    return signedXdr;
  } catch (error) {
    console.error("Error signing transaction with Freighter:", error);
  }
}
```

## 6. Transaction Submission

Finally, submit the signed transaction back to the network.

```javascript
import { SorobanRpc } from "@stellar/stellar-sdk";

async function submitTransaction(signedXdr) {
  const server = new SorobanRpc.Server("https://soroban-testnet.stellar.org");
  const tx = StellarSdk.TransactionBuilder.fromXDR(signedXdr, StellarSdk.Networks.TESTNET);
  
  try {
    const response = await server.sendTransaction(tx);
    console.log("Transaction submitted successfully!", response);
  } catch (error) {
    console.error("Error submitting transaction:", error);
  }
}
```

## 7. Error Handling

It is essential to handle errors effectively. Users may reject the transaction or there might be issues with network connectivity.

- **User Rejected Action**: Always catch exceptions thrown by `requestAccess` or `signTransaction`. Freighter throws specific errors when a user rejects a prompt.
- **Simulation Failure**: The simulation phase often reveals contract logic errors. Check the simulation response carefully before signing.
- **Submission Failure**: Transactions can fail after submission due to low fees or sequence number mismatches.

## Testing Guide

To test Freighter locally during development:
1. Enable developer mode in your Freighter settings.
2. Select the Testnet network or add a custom network pointing to your local node.
3. Ensure the active account has sufficient testnet/local XLM for fees.

## Troubleshooting

- **`isConnected()` returns false**: Ensure the Freighter extension is installed and active in the browser.
- **"User declined access" error**: The user closed the prompt or explicitly clicked "Reject". Prompt the user nicely to approve the request if it's necessary for functionality.
- **Transaction fails with sequence error**: Make sure you have fetched the latest account state from the RPC server immediately before building the transaction.
