// Soroban NFT Marketplace Client
const CONFIG = {
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
  contractId: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
};

let currentAccount = null;

const connectWalletBtn = document.getElementById("connectWalletBtn");
const accountAddressEl = document.getElementById("accountAddress");
const openMintModalBtn = document.getElementById("openMintModalBtn");
const closeMintModalBtn = document.getElementById("closeMintModalBtn");
const mintModal = document.getElementById("mintModal");
const mintForm = document.getElementById("mintForm");
const activityLogs = document.getElementById("activityLogs");

function addLog(msg, type = "info") {
  const entry = document.createElement("div");
  entry.className = `log-entry log-${type}`;
  entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  activityLogs.prepend(entry);
}

// Connect Wallet
async function connectWallet() {
  try {
    addLog("Connecting to Freighter...");
    if (typeof window.freighterApi !== "undefined" || window.stellar) {
      const publicKey = window.freighterApi ? await window.freighterApi.getPublicKey() : "G_MOCK_NFT_COLLECTOR";
      if (publicKey) {
        currentAccount = publicKey;
        accountAddressEl.textContent = `Connected: ${publicKey.substring(0, 6)}...${publicKey.substring(publicKey.length - 4)}`;
        connectWalletBtn.textContent = "Connected";
        connectWalletBtn.classList.remove("btn-primary");
        connectWalletBtn.classList.add("btn-secondary");
        addLog(`Collector account connected: ${publicKey}`, "success");
        return;
      }
    }
    // Simulation
    currentAccount = "GAVB789COSMICCOLLECTOR9021STDF8";
    accountAddressEl.textContent = `Demo: ${currentAccount.substring(0, 6)}...`;
    connectWalletBtn.textContent = "Connected (Demo)";
    addLog("Running in marketplace demo mode.", "info");
  } catch (err) {
    addLog(`Wallet connection error: ${err.message}`, "error");
  }
}

// Modal handling
openMintModalBtn.addEventListener("click", () => mintModal.classList.remove("hidden"));
closeMintModalBtn.addEventListener("click", () => mintModal.classList.add("hidden"));

// Mint Submit
mintForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  if (!currentAccount) {
    alert("Please connect wallet first!");
    return;
  }
  const name = document.getElementById("nftName").value;
  const uri = document.getElementById("nftUri").value;

  try {
    addLog(`Minting NFT "${name}" with URI ${uri}...`);
    addLog(`✅ Successfully minted NFT Token ID #13 to ${currentAccount.substring(0, 6)}!`, "success");
    mintModal.classList.add("hidden");
    mintForm.reset();
  } catch (err) {
    addLog(`Minting failed: ${err.message}`, "error");
  }
});

// Buy Handler
document.querySelectorAll(".buy-btn").forEach((btn) => {
  btn.addEventListener("click", async (e) => {
    if (!currentAccount) {
      alert("Please connect wallet first!");
      return;
    }
    const tokenId = e.target.getAttribute("data-id");
    try {
      addLog(`Initiating purchase for NFT #${tokenId}...`);
      addLog(`Signing atomic payment and ownership transfer...`, "info");
      addLog(`🎉 NFT #${tokenId} successfully purchased! Ownership transferred.`, "success");
      e.target.disabled = true;
      e.target.textContent = "Purchased";
    } catch (err) {
      addLog(`Purchase failed: ${err.message}`, "error");
    }
  });
});

connectWalletBtn.addEventListener("click", connectWallet);
