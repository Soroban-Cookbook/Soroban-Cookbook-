/**
 * Soroban Cookbook — Privacy-Respecting Documentation Analytics
 *
 * Tracks three signals required by Issue #927:
 *   1. Page views          — fired on every page load / SPA navigation
 *   2. Search queries      — fired when the user runs an mdBook search
 *   3. User paths          — the sequence of pages visited in a session
 *
 * Privacy guarantees
 *   - No cookies.  Path history lives in `sessionStorage` (per-tab, cleared on close).
 *   - No personally identifiable information is collected.  The session id is a
 *     random 8-hex string generated per tab — it cannot be correlated across
 *     sessions or devices.
 *   - Respects the Do-Not-Track header (`navigator.doNotTrack`).
 *   - All data is sent via `navigator.sendBeacon` (fire-and-forget, no blocking).
 *   - If no endpoint is configured the script silently no-ops, so it is safe to
 *     ship to production before a collection backend exists.
 *
 * Configuration
 *   Set `window.SOROBAN_ANALYTICS_ENDPOINT` to the beacon URL before this script
 *   loads (e.g. in a tiny inline script in the mdBook theme header, or via the
 *   `additional-js` / theme override).  When unset the script records locally
 *   only and exposes `window.SorobanAnalytics.exportSession()` for debugging.
 *
 * Related
 *   - Metric definitions:  docs/community-metrics.md § 3.6
 *   - Dashboard:            docs/documentation-metrics.md
 *   - Collection workflow:  .github/workflows/documentation-metrics.yml
 */
(function () {
  "use strict";

  // --- Respect Do-Not-Track ----------------------------------------------
  var dnt =
    navigator.doNotTrack || window.doNotTrack || navigator.msDoNotTrack;
  if (dnt === "1" || dnt === "yes") {
    return; // user opted out — load nothing
  }

  // --- Configuration -----------------------------------------------------
  var ENDPOINT = window.SOROBAN_ANALYTICS_ENDPOINT || "";
  var SITE = "soroban-cookbook-mdbook";
  var SESSION_KEY = "soroban-analytics-session";
  var PATH_KEY = "soroban-analytics-path";
  var MAX_PATH_LENGTH = 50; // cap recorded path steps

  // --- Anonymous session id (per-tab, non-persistent) --------------------
  var sessionId;
  try {
    sessionId = sessionStorage.getItem(SESSION_KEY);
    if (!sessionId) {
      // Random 8-hex string — not a fingerprint, just a tab-scoped correlation id
      var rand = new Uint32Array(1);
      (window.crypto || {}).getRandomValues
        ? crypto.getRandomValues(rand)
        : (rand[0] = (Math.random() * 0xffffffff) >>> 0);
      sessionId = rand[0].toString(16).padStart(8, "0");
      sessionStorage.setItem(SESSION_KEY, sessionId);
    }
  } catch (e) {
    sessionId = "00000000"; // sessionStorage unavailable (private mode) — still works
  }

  // --- User path helpers -------------------------------------------------
  function getPath() {
    try {
      var raw = sessionStorage.getItem(PATH_KEY);
      return raw ? JSON.parse(raw) : [];
    } catch (e) {
      return [];
    }
  }

  function appendPath(path) {
    var p = getPath();
    if (p[p.length - 1] !== path) {
      p.push(path);
      if (p.length > MAX_PATH_LENGTH) p = p.slice(-MAX_PATH_LENGTH);
      try {
        sessionStorage.setItem(PATH_KEY, JSON.stringify(p));
      } catch (e) {
        /* ignore */
      }
    }
    return p;
  }

  // --- Beacon helper -----------------------------------------------------
  function send(event) {
    event.site = SITE;
    event.session = sessionId;
    event.path = location.pathname + location.hash;
    event.referrer = document.referrer || null;
    event.ts = new Date().toISOString();

    // Fire-and-forget via sendBeacon (does not block page unload)
    if (ENDPOINT && navigator.sendBeacon) {
      try {
        navigator.sendBeacon(
          ENDPOINT,
          new Blob([JSON.stringify(event)], { type: "application/json" })
        );
      } catch (e) {
        /* network failures are non-fatal */
      }
    }
  }

  // --- 1. Page view tracking ---------------------------------------------
  function trackPageView() {
    var current = location.pathname;
    var path = appendPath(current);

    send({
      type: "pageview",
      title: document.title || "",
      pathLength: path.length,
      // Include a compact representation of the path so the backend can
      // reconstruct user journeys without storing per-tab state server-side.
      userPath: path.slice(-10), // last 10 pages — enough to analyse flows
    });
  }

  // --- 2. Search query tracking ------------------------------------------
  // mdBook's built-in search uses an <input id="searchbar">.  We debounce so
  // we capture the *submitted* query, not every keystroke.
  var searchTimer = null;
  var lastQuery = "";

  function setupSearchTracking() {
    var searchInput = document.getElementById("searchbar");
    if (!searchInput || searchInput.dataset.analyticsBound) return;
    searchInput.dataset.analyticsBound = "1";

    searchInput.addEventListener("input", function () {
      if (searchTimer) clearTimeout(searchTimer);
      var query = searchInput.value.trim();
      if (query.length < 2 || query === lastQuery) return;
      searchTimer = setTimeout(function () {
        lastQuery = query;
        send({ type: "search", query: query });
      }, 800); // debounce 800ms — captures the final query after typing stops
    });

    // Also fire on Enter / form submit
    searchInput.addEventListener("keydown", function (e) {
      if (e.key === "Enter") {
        var query = searchInput.value.trim();
        if (query.length >= 2 && query !== lastQuery) {
          lastQuery = query;
          send({ type: "search", query: query });
        }
      }
    });
  }

  // --- SPA navigation detection -----------------------------------------
  // mdBook does full page loads by default, but theme overrides may use
  // pushState.  We patch history methods + listen to popstate to cover both.
  var origPush = history.pushState;
  var origReplace = history.replaceState;

  history.pushState = function () {
    origPush.apply(history, arguments);
    setTimeout(trackPageView, 0);
  };
  history.replaceState = function () {
    origReplace.apply(history, arguments);
    setTimeout(trackPageView, 0);
  };
  window.addEventListener("popstate", function () {
    setTimeout(trackPageView, 0);
  });

  // --- Boot ---------------------------------------------------------------
  function boot() {
    trackPageView();
    setupSearchTracking();
    // Re-bind search after mdBook re-renders the sidebar (MutationObserver
    // catches DOM updates from theme overrides).
    var observer = new MutationObserver(function () {
      setupSearchTracking();
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  // --- Public debugging API ----------------------------------------------
  window.SorobanAnalytics = {
    exportSession: function () {
      return {
        sessionId: sessionId,
        path: getPath(),
        endpoint: ENDPOINT || "(not configured)",
      };
    },
  };
})();
