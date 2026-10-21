# Mobile Testing Guide

How to test the Soroban Cookbook web frontend — and dApps built the same way
— on mobile browsers and mobile wallets.

This repository doesn't ship a native mobile app; "mobile testing" here means
verifying that the [webapp](../webapp) playground and any Soroban dApp built
against it work correctly on a phone: the page has to render usably on a
small screen, and wallet connections have to work through the mobile wallet
flows described in the [Wallet Ecosystem Survey](../book/src/docs/wallet-ecosystem.md)
and [Wallet Integration Guide](../book/src/docs/wallet-integration.md).

---

## Why This Matters

Several wallets in the Stellar ecosystem are mobile-first or mobile-only for
end users — xBull and Lobstr both ship mobile apps, and users reach dApps
through a mobile browser far more often than through a desktop extension.
A dApp that only gets tested with a Freighter browser extension on desktop
can ship with a broken layout or a wallet flow that never completes on the
device most retail users actually have.

## Responsive Testing Checklist

Test the webapp (or any page built on it) at these breakpoints before
merging a UI change:

| Viewport | Represents |
| --- | --- |
| 360×640 | Small Android phones |
| 390×844 | Modern iPhone (iOS Safari) |
| 768×1024 | Tablet / iPad portrait |

Check on each breakpoint:

- No horizontal scroll or clipped content
- Buttons and links are large enough to tap (44×44px minimum touch target)
- Modals/dialogs (e.g. a wallet-connect prompt) stay within the viewport and are dismissible
- Text remains readable without zooming

### Local Setup

```bash
cd webapp
bun install
bun run dev
```

Open the dev server in your browser's device emulation mode (Chrome DevTools
device toolbar or Firefox Responsive Design Mode) and step through the
breakpoints above. For a final check before a release, test on at least one
real iOS and one real Android device — emulators don't reproduce mobile
Safari's viewport quirks or a mobile wallet app's in-app browser behavior.

## Testing Mobile Wallet Connections

Mobile wallet connections typically go through **WalletConnect v2** rather
than a browser extension (see [Standards Compliance](../book/src/docs/wallet-ecosystem.md#3-standards-compliance)
in the wallet ecosystem survey). Verify the full flow end to end:

1. **Connect** — open the dApp on desktop, trigger the wallet-connect flow,
   and scan the resulting QR code with the mobile wallet app (xBull or
   Lobstr). Confirm the session pairs successfully.
2. **Sign** — trigger a transaction from the dApp and confirm the mobile
   wallet displays a clear signing prompt with the correct network
   (Testnet vs. Mainnet — see [Network Configuration](./deployment.md#network-configuration)).
3. **Reject** — decline the signing prompt on the mobile wallet and confirm
   the dApp surfaces a clear "user rejected" state rather than hanging.
4. **Reconnect** — background the wallet app or lock the phone mid-session,
   then return to the dApp and confirm it recovers or clearly asks the user
   to reconnect rather than silently failing.
5. **Wrong network** — switch the mobile wallet to the wrong network before
   signing, and confirm the dApp catches the mismatch instead of submitting
   a doomed transaction.

For automated coverage, follow the same "mock the wallet provider" approach
described in the [Wallet Integration Guide's Testing Guide](../book/src/docs/wallet-integration.md#testing-guide) —
mock at the provider boundary rather than depending on a real mobile device
in CI.

## Device & Browser Matrix

Minimum manual QA coverage before a frontend release:

| Platform | Browser | Priority |
| --- | --- | --- |
| iOS | Safari | Critical — iOS only allows WebKit-based browsers |
| Android | Chrome | Critical — largest mobile user base |
| Android | Samsung Internet | Recommended if analytics show meaningful traffic |
| iOS / Android | In-app browser of the wallet app (xBull, Lobstr) | Critical for WalletConnect deep-link flows |

## Common Mobile-Specific Issues

- **Viewport meta tag missing or wrong** — causes desktop-scaled rendering on
  phones. Confirm `<meta name="viewport" content="width=device-width, initial-scale=1">`
  is present in the app's layout.
- **QR code too small to scan** — WalletConnect QR modals need to render at a
  large enough size on small screens; test the modal at the 360×640
  breakpoint specifically.
- **Deep links not returning to the browser tab** — after signing in a wallet
  app opened via deep link, mobile OSes don't always return focus to the
  originating browser tab automatically; verify your app's reconnect/resume
  logic handles this.
- **Fixed-position elements covering content** — sticky headers/footers can
  overlap content on short mobile viewports; check with the on-screen
  keyboard open, which shrinks the visible viewport further.

## Related Guides

- [Wallet Ecosystem Survey](../book/src/docs/wallet-ecosystem.md) — which wallets support mobile and how
- [Wallet Integration Guide](../book/src/docs/wallet-integration.md) — connecting and signing, including the existing Testing Guide section
- [Deployment Guide](./deployment.md) — network configuration referenced above
