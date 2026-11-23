/**
 * Soroban Cookbook — Webapp Analytics Module
 *
 * Privacy-respecting analytics for the Next.js webapp (Issue #927).
 * Tracks page views, search queries, and user paths without cookies or
 * personally identifiable information.
 *
 * Privacy guarantees
 *   - No cookies.  Path history lives in `sessionStorage` (per-tab, cleared on close).
 *   - No PII.  Session id is a random 8-hex string per tab.
 *   - Respects Do-Not-Track (`navigator.doNotTrack`).
 *   - Uses `navigator.sendBeacon` (fire-and-forget, non-blocking).
 *   - No-ops silently when the endpoint is not configured.
 *
 * Usage
 *   import { trackPageView, trackSearch, initAnalytics } from "@/lib/analytics";
 *   initAnalytics();                          // in layout or a client component
 *   trackPageView("/dashboard");              // on route change
 *   trackSearch("token vesting");             // on search submit
 */

const SITE = "soroban-cookbook-webapp";
const SESSION_KEY = "soroban-analytics-session";
const PATH_KEY = "soroban-analytics-path";
const MAX_PATH_LENGTH = 50;

// --- Do-Not-Track check ---------------------------------------------------
function isDNT(): boolean {
  if (typeof navigator === "undefined") return false;
  const dnt =
    navigator.doNotTrack || (navigator as any).doNotTrack || (navigator as any).msDoNotTrack;
  return dnt === "1" || dnt === "yes";
}

// --- Endpoint configuration -----------------------------------------------
function getEndpoint(): string {
  // Can be set at build time via NEXT_PUBLIC_ANALYTICS_ENDPOINT or at runtime
  // via window.SOROBAN_ANALYTICS_ENDPOINT (injected in <head>).
  if (typeof window !== "undefined" && (window as any).SOROBAN_ANALYTICS_ENDPOINT) {
    return (window as any).SOROBAN_ANALYTICS_ENDPOINT as string;
  }
  return process.env.NEXT_PUBLIC_ANALYTICS_ENDPOINT || "";
}

// --- Session id (per-tab, anonymous) ---------------------------------------
let cachedSessionId: string | null = null;

function getSessionId(): string {
  if (cachedSessionId) return cachedSessionId;
  try {
    const stored = sessionStorage.getItem(SESSION_KEY);
    if (stored) {
      cachedSessionId = stored;
      return stored;
    }
    // Generate random 8-hex string
    const rand = new Uint32Array(1);
    if (typeof crypto !== "undefined" && crypto.getRandomValues) {
      crypto.getRandomValues(rand);
    } else {
      rand[0] = (Math.random() * 0xffffffff) >>> 0;
    }
    const id = rand[0].toString(16).padStart(8, "0");
    sessionStorage.setItem(SESSION_KEY, id);
    cachedSessionId = id;
    return id;
  } catch {
    cachedSessionId = "00000000";
    return cachedSessionId;
  }
}

// --- User path helpers -----------------------------------------------------
function getPath(): string[] {
  try {
    const raw = sessionStorage.getItem(PATH_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function appendPath(path: string): string[] {
  const p = getPath();
  if (p[p.length - 1] !== path) {
    p.push(path);
    const capped = p.slice(-MAX_PATH_LENGTH);
    try {
      sessionStorage.setItem(PATH_KEY, JSON.stringify(capped));
    } catch {
      /* ignore */
    }
    return capped;
  }
  return p;
}

// --- Beacon sender ---------------------------------------------------------
interface AnalyticsEvent {
  type: string;
  site: string;
  session: string;
  path: string;
  referrer: string | null;
  ts: string;
  [key: string]: any;
}

function send(event: { type: string; [key: string]: any }): void {
  if (isDNT()) return;
  const endpoint = getEndpoint();
  if (!endpoint || typeof navigator === "undefined") return;

  const fullEvent: AnalyticsEvent = {
    ...event,
    site: SITE,
    session: getSessionId(),
    path: typeof window !== "undefined" ? window.location.pathname + window.location.hash : "",
    referrer: typeof document !== "undefined" ? document.referrer || null : null,
    ts: new Date().toISOString(),
  };

  try {
    navigator.sendBeacon(
      endpoint,
      new Blob([JSON.stringify(fullEvent)], { type: "application/json" })
    );
  } catch {
    /* network failures are non-fatal */
  }
}

// --- Public API ------------------------------------------------------------

/**
 * Track a page view.  Call this on route changes (e.g. in a useEffect
 * watching `usePathname()` from `next/navigation`).
 */
export function trackPageView(pathOverride?: string): void {
  const current = pathOverride || (typeof window !== "undefined" ? window.location.pathname : "");
  if (!current) return;
  const path = appendPath(current);
  send({
    type: "pageview",
    title: typeof document !== "undefined" ? document.title : "",
    pathLength: path.length,
    userPath: path.slice(-10),
  });
}

/**
 * Track a search query.  Call this when the user submits a search.
 */
export function trackSearch(query: string): void {
  const trimmed = query.trim();
  if (trimmed.length < 2) return;
  send({ type: "search", query: trimmed });
}

/**
 * Track a custom event (e.g. filter applied, link clicked).
 */
export function trackEvent(name: string, properties?: Record<string, any>): void {
  send({ type: "event", event: name, ...properties });
}

/**
 * Export the current session data for debugging.
 */
export function exportSession(): {
  sessionId: string;
  path: string[];
  endpoint: string;
} {
  return {
    sessionId: getSessionId(),
    path: getPath(),
    endpoint: getEndpoint() || "(not configured)",
  };
}

/**
 * Initialise analytics — should be called once on app mount (client-side).
 * Optionally accepts a path to track the initial page view.
 */
export function initAnalytics(): void {
  if (typeof window === "undefined") return; // server-side no-op
  if (isDNT()) return;
  trackPageView();
}
