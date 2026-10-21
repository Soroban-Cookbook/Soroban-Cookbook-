/**
 * Soroban Cookbook Documentation Analytics
 *
 * Tracks page views, search queries, user navigation patterns,
 * and engagement metrics for the mdBook documentation.
 */

(function() {
    'use strict';

    /**
     * Initialize analytics on page load
     */
    class DocumentationAnalytics {
        constructor() {
            this.sessionId = this.generateSessionId();
            this.userId = this.getUserId();
            this.pageStartTime = Date.now();

            this.init();
        }

        /**
         * Initialize analytics tracking
         */
        init() {
            // Track page view
            this.trackPageView();

            // Track link clicks
            this.trackLinkClicks();

            // Track scroll depth
            this.trackScrollDepth();

            // Track time on page
            window.addEventListener('beforeunload', () => {
                this.trackTimeOnPage();
            });

            // Track search if search exists
            this.trackSearch();
        }

        /**
         * Track page view event
         */
        trackPageView() {
            const event = {
                type: 'page_view',
                page: window.location.pathname,
                title: document.title,
                referrer: document.referrer,
                timestamp: new Date().toISOString(),
                sessionId: this.sessionId,
                userId: this.userId,
            };

            this.sendEvent(event);
        }

        /**
         * Track link clicks
         */
        trackLinkClicks() {
            document.addEventListener('click', (e) => {
                const link = e.target.closest('a');
                if (!link) return;

                const href = link.getAttribute('href');

                // Track internal links
                if (href && !href.startsWith('http') && !href.startsWith('//')) {
                    const event = {
                        type: 'link_click',
                        href: href,
                        text: link.textContent,
                        timestamp: new Date().toISOString(),
                        sessionId: this.sessionId,
                    };
                    this.sendEvent(event);
                }
            });
        }

        /**
         * Track scroll depth
         */
        trackScrollDepth() {
            const depths = [25, 50, 75, 100];
            const tracked = new Set();

            window.addEventListener('scroll', () => {
                const scrollPercentage = (
                    (window.scrollY + window.innerHeight) /
                    document.documentElement.scrollHeight
                ) * 100;

                depths.forEach(depth => {
                    if (scrollPercentage >= depth && !tracked.has(depth)) {
                        tracked.add(depth);

                        const event = {
                            type: 'scroll_depth',
                            page: window.location.pathname,
                            depth: depth,
                            timestamp: new Date().toISOString(),
                            sessionId: this.sessionId,
                        };
                        this.sendEvent(event);
                    }
                });
            });
        }

        /**
         * Track time on page
         */
        trackTimeOnPage() {
            const timeOnPage = (Date.now() - this.pageStartTime) / 1000;

            const event = {
                type: 'page_engagement',
                page: window.location.pathname,
                timeOnPage: Math.round(timeOnPage),
                timestamp: new Date().toISOString(),
                sessionId: this.sessionId,
            };

            this.sendEvent(event);
        }

        /**
         * Track search queries
         */
        trackSearch() {
            const searchInput = document.querySelector('input[type="search"]');
            if (!searchInput) return;

            let searchTimeout;
            let lastQuery = '';

            searchInput.addEventListener('input', (e) => {
                clearTimeout(searchTimeout);
                const query = e.target.value;

                if (query.length > 2 && query !== lastQuery) {
                    lastQuery = query;

                    searchTimeout = setTimeout(() => {
                        const event = {
                            type: 'search',
                            query: query,
                            timestamp: new Date().toISOString(),
                            sessionId: this.sessionId,
                        };
                        this.sendEvent(event);
                    }, 500);
                }
            });
        }

        /**
         * Send event to analytics backend
         */
        async sendEvent(event) {
            try {
                // For now, log to console in development
                if (window.location.hostname === 'localhost') {
                    console.log('[Analytics]', event);
                    return;
                }

                // In production, send to analytics endpoint
                await fetch('/api/analytics', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(event),
                }).catch(() => {
                    // Silently fail - don't break page
                });
            } catch (error) {
                // Silently fail
            }
        }

        /**
         * Generate unique session ID
         */
        generateSessionId() {
            return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        }

        /**
         * Get or create user ID
         */
        getUserId() {
            let userId = localStorage.getItem('doc_user_id');
            if (!userId) {
                userId = `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
                try {
                    localStorage.setItem('doc_user_id', userId);
                } catch (e) {
                    // localStorage might be disabled
                }
            }
            return userId;
        }
    }

    /**
     * Initialize analytics when DOM is ready
     */
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            new DocumentationAnalytics();
        });
    } else {
        new DocumentationAnalytics();
    }

    /**
     * Google Analytics integration (if GA_ID is set)
     */
    if (typeof window.GA_ID !== 'undefined') {
        window.dataLayer = window.dataLayer || [];

        function gtag() {
            dataLayer.push(arguments);
        }

        gtag('js', new Date());
        gtag('config', window.GA_ID);
    }
})();
