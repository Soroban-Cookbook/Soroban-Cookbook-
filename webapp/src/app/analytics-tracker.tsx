"use client";

import { useEffect } from "react";
import { usePathname } from "next/navigation";
import { initAnalytics, trackPageView } from "@/lib/analytics";

/**
 * Client component that initialises privacy-respecting analytics and tracks
 * page views on route changes. Mounted from the root layout (which must stay
 * a Server Component to export `metadata`).
 *
 * Issue #927 — Track Documentation Metrics.
 */
export default function AnalyticsTracker() {
  const pathname = usePathname();

  useEffect(() => {
    initAnalytics();
  }, []);

  useEffect(() => {
    trackPageView(pathname ?? undefined);
  }, [pathname]);

  return null; // renders nothing — purely a side-effect component
}
