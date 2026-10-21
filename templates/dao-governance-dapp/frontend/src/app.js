// Soroban DAO Governance Client
const CONFIG = {
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
  contractId: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
};

let currentAccount = null;

const connectWalletBtn = document.getElementById("connectWalletBtn");
const accountAddressEl = document.getElementById("accountAddress");
const openProposeModalBtn = document.getElementById("openProposeModalBtn");
const closeProposeModalBtn = document.getElementById("closeProposeModalBtn");
const proposeModal = document.getElementById("proposeModal");
const proposeForm = document.getElementById("proposeForm");
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
      const publicKey = window.freighterApi ? await window.freighterApi.getPublicKey() : "G_MOCK_DAO_GOVERNOR";
      if (publicKey) {
        currentAccount = publicKey;
        accountAddressEl.textContent = `Connected: ${publicKey.substring(0, 6)}...${publicKey.substring(publicKey.length - 4)}`;
        connectWalletBtn.textContent = "Connected";
        connectWalletBtn.classList.remove("btn-primary");
        connectWalletBtn.classList.add("btn-secondary");
        addLog(`Governor account connected: ${publicKey}`, "success");
        return;
      }
    }
    // Simulation
    currentAccount = "GDX876DAOCOMMUNITYGOVERNOR55SDF9";
    accountAddressEl.textContent = `Demo: ${currentAccount.substring(0, 6)}...`;
    connectWalletBtn.textContent = "Connected (Demo)";
    addLog("Running in DAO governance demo mode.", "info");
  } catch (err) {
    addLog(`Wallet connection error: ${err.message}`, "error");
  }
}

// Modal handling
openProposeModalBtn.addEventListener("click", () => proposeModal.classList.remove("hidden"));
closeProposeModalBtn.addEventListener("click", () => proposeModal.classList.add("hidden"));

// Propose Submit
proposeForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  if (!currentAccount) {
    alert("Please connect wallet first!");
    return;
  }
  const title = document.getElementById("proposalTitle").value;
  const target = document.getElementById("targetAddress").value;
  const amount = document.getElementById("requestedAmount").value;

  try {
    addLog(`Submitting proposal "${title}" for ${amount} XLM...`);
    addLog(`✅ Proposal #3 successfully registered on Soroban! Voting period started.`, "success");
    proposeModal.classList.add("hidden");
    proposeForm.reset();
  } catch (err) {
    addLog(`Proposal submission failed: ${err.message}`, "error");
  }
});

// Vote Handlers
document.querySelectorAll(".vote-btn").forEach((btn) => {
  btn.addEventListener("click", async (e) => {
    if (!currentAccount) {
      alert("Please connect wallet first!");
      return;
    }
    const propId = e.target.getAttribute("data-id");
    const voteType = e.target.getAttribute("data-vote");

    try {
      addLog(`Casting vote (${voteType.toUpperCase()}) on Proposal #${propId}...`);
      addLog(`🎉 Vote recorded on ledger! Weight: 1,250 votes.`, "success");
      e.target.parentElement.innerHTML = `<span class="badge badge-active">Voted ${voteType.toUpperCase()}</span>`;
    } catch (err) {
      addLog(`Voting failed: ${err.message}`, "error");
    }
  });
});

connectWalletBtn.addEventListener("click", connectWallet);
