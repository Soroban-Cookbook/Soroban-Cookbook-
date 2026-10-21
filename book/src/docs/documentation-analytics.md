# Documentation Analytics Setup

## Overview

This document describes the analytics infrastructure for tracking documentation usage, page views, search queries, and user navigation patterns across the Soroban Cookbook documentation site.

## Analytics Infrastructure

### Components

1. **Page View Tracking**: Monitors which documentation pages are accessed
2. **Search Analytics**: Tracks search queries and their success rates
3. **User Path Analysis**: Analyzes how users navigate through documentation
4. **Performance Metrics**: Measures page load times and responsiveness
5. **Dashboard**: Visualizes analytics data for insights

## Implementation

### Google Analytics Integration

#### Configuration

Add Google Analytics to your mdBook configuration in `book.toml`:

```toml
[output.html]
additional-js = ["analytics.js"]

[build]
extra-css = ["analytics.css"]
```

#### Analytics Script

Create `book/theme/analytics.js`:

```javascript
// Google Analytics Configuration
window.dataLayer = window.dataLayer || [];

function gtag() {
    dataLayer.push(arguments);
}

gtag('js', new Date());
gtag('config', 'GA_MEASUREMENT_ID');

// Page View Tracking
document.addEventListener('DOMContentLoaded', function() {
    gtag('event', 'page_view', {
        page_path: window.location.pathname,
        page_title: document.title,
    });
});

// Track Search Events
function trackSearch(query, resultsFound) {
    gtag('event', 'search', {
        search_term: query,
        results_found: resultsFound,
    });
}

// Track Time on Page
let pageStartTime = Date.now();

window.addEventListener('beforeunload', function() {
    const timeOnPage = (Date.now() - pageStartTime) / 1000;
    gtag('event', 'page_engagement', {
        page_path: window.location.pathname,
        time_on_page: Math.round(timeOnPage),
    });
});
```

### Custom Analytics Collection

#### Local Analytics Service

Implement a custom analytics collector for detailed tracking:

```javascript
class DocumentationAnalytics {
    constructor(config) {
        this.config = config;
        this.sessionId = this.generateSessionId();
        this.userId = this.getUserId();
    }
    
    // Track page view
    trackPageView(page) {
        const event = {
            type: 'page_view',
            page: page,
            timestamp: new Date().toISOString(),
            sessionId: this.sessionId,
            userId: this.userId,
        };
        this.sendEvent(event);
    }
    
    // Track search
    trackSearch(query, resultsCount) {
        const event = {
            type: 'search',
            query: query,
            results_count: resultsCount,
            timestamp: new Date().toISOString(),
            sessionId: this.sessionId,
        };
        this.sendEvent(event);
    }
    
    // Track link clicks
    trackLinkClick(target, linkText) {
        const event = {
            type: 'link_click',
            target: target,
            link_text: linkText,
            timestamp: new Date().toISOString(),
            sessionId: this.sessionId,
        };
        this.sendEvent(event);
    }
    
    // Track scroll depth
    trackScrollDepth(page, percentage) {
        const event = {
            type: 'scroll_depth',
            page: page,
            depth_percentage: percentage,
            timestamp: new Date().toISOString(),
            sessionId: this.sessionId,
        };
        this.sendEvent(event);
    }
    
    // Track code examples executed
    trackCodeExample(exampleId, status) {
        const event = {
            type: 'code_example',
            example_id: exampleId,
            status: status, // 'success', 'error', 'viewed'
            timestamp: new Date().toISOString(),
            sessionId: this.sessionId,
        };
        this.sendEvent(event);
    }
    
    // Send event to analytics service
    async sendEvent(event) {
        try {
            await fetch('/api/analytics', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(event),
            });
        } catch (error) {
            console.error('Failed to send analytics event:', error);
        }
    }
    
    generateSessionId() {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }
    
    getUserId() {
        let userId = localStorage.getItem('doc_user_id');
        if (!userId) {
            userId = `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
            localStorage.setItem('doc_user_id', userId);
        }
        return userId;
    }
}
```

## Key Metrics Tracked

### Page Metrics

- **Page Views**: Total visits to each documentation page
- **Unique Visitors**: Number of unique users per page
- **Time on Page**: Average time spent on each page
- **Bounce Rate**: Percentage of users who leave without further interaction
- **Entry/Exit Pages**: Where users start and end their sessions

### Search Metrics

- **Search Queries**: What users are searching for
- **Search Success Rate**: Percentage of searches returning results
- **Popular Search Terms**: Most frequently searched topics
- **Search-to-Page Conversion**: When users search then view a page

### Navigation Metrics

- **User Paths**: Common navigation patterns
- **Link Click Tracking**: Which links are most frequently clicked
- **Section Popularity**: Which documentation sections get the most traffic
- **Drop-off Points**: Where users stop navigating

### Code Example Metrics

- **Examples Viewed**: Which code examples are accessed
- **Copy-to-Clipboard**: When users copy example code
- **Syntax Highlighting**: Does syntax highlighting improve engagement?

## Analytics Dashboard

### Dashboard Implementation

Create a dashboard at `/docs/analytics-dashboard.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Documentation Analytics Dashboard</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        .dashboard {
            max-width: 1400px;
            margin: 0 auto;
        }
        .header {
            margin-bottom: 30px;
        }
        .metric-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .metric-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .metric-value {
            font-size: 32px;
            font-weight: bold;
            color: #2c3e50;
        }
        .metric-label {
            color: #7f8c8d;
            font-size: 14px;
            margin-top: 8px;
        }
        .chart-container {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 20px;
            position: relative;
            height: 400px;
        }
        h1 {
            color: #2c3e50;
            margin: 0;
        }
        .subtitle {
            color: #7f8c8d;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="dashboard">
        <div class="header">
            <h1>Documentation Analytics</h1>
            <p class="subtitle">Last 30 days</p>
        </div>
        
        <div class="metric-grid">
            <div class="metric-card">
                <div class="metric-value" id="total-views">0</div>
                <div class="metric-label">Total Page Views</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="unique-visitors">0</div>
                <div class="metric-label">Unique Visitors</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="avg-session-duration">0m</div>
                <div class="metric-label">Avg Session Duration</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="bounce-rate">0%</div>
                <div class="metric-label">Bounce Rate</div>
            </div>
        </div>
        
        <div class="chart-container">
            <canvas id="page-views-chart"></canvas>
        </div>
        
        <div class="chart-container">
            <canvas id="top-pages-chart"></canvas>
        </div>
        
        <div class="chart-container">
            <canvas id="search-queries-chart"></canvas>
        </div>
    </div>
    
    <script>
        // Initialize dashboard data
        async function initializeDashboard() {
            // Fetch analytics data from API
            const data = await fetch('/api/analytics/summary').then(r => r.json());
            
            // Update metrics
            document.getElementById('total-views').textContent = 
                data.total_views.toLocaleString();
            document.getElementById('unique-visitors').textContent = 
                data.unique_visitors.toLocaleString();
            document.getElementById('avg-session-duration').textContent = 
                `${Math.round(data.avg_session_duration / 60)}m`;
            document.getElementById('bounce-rate').textContent = 
                `${Math.round(data.bounce_rate)}%`;
            
            // Initialize charts
            initializePageViewsChart(data.page_views_over_time);
            initializeTopPagesChart(data.top_pages);
            initializeSearchQueriesChart(data.search_queries);
        }
        
        function initializePageViewsChart(data) {
            const ctx = document.getElementById('page-views-chart').getContext('2d');
            new Chart(ctx, {
                type: 'line',
                data: {
                    labels: data.map(d => d.date),
                    datasets: [{
                        label: 'Page Views',
                        data: data.map(d => d.views),
                        borderColor: '#3498db',
                        backgroundColor: 'rgba(52, 152, 219, 0.1)',
                        tension: 0.4,
                    }],
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        title: {
                            display: true,
                            text: 'Page Views Over Time',
                        },
                    },
                },
            });
        }
        
        function initializeTopPagesChart(data) {
            const ctx = document.getElementById('top-pages-chart').getContext('2d');
            new Chart(ctx, {
                type: 'bar',
                data: {
                    labels: data.map(d => d.page),
                    datasets: [{
                        label: 'Views',
                        data: data.map(d => d.views),
                        backgroundColor: '#2ecc71',
                    }],
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    indexAxis: 'y',
                    plugins: {
                        title: {
                            display: true,
                            text: 'Top 10 Pages',
                        },
                    },
                },
            });
        }
        
        function initializeSearchQueriesChart(data) {
            const ctx = document.getElementById('search-queries-chart').getContext('2d');
            new Chart(ctx, {
                type: 'bar',
                data: {
                    labels: data.map(d => d.query),
                    datasets: [{
                        label: 'Searches',
                        data: data.map(d => d.count),
                        backgroundColor: '#e74c3c',
                    }],
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        title: {
                            display: true,
                            text: 'Top Search Queries',
                        },
                    },
                },
            });
        }
        
        // Initialize dashboard on page load
        document.addEventListener('DOMContentLoaded', initializeDashboard);
    </script>
</body>
</html>
```

## API Endpoints

### Analytics Collection Endpoint

**POST /api/analytics**

Collect analytics events from the frontend:

```json
{
    "type": "page_view",
    "page": "/getting-started.html",
    "timestamp": "2024-01-15T10:30:00Z",
    "sessionId": "session_12345",
    "userId": "user_abcde"
}
```

### Analytics Summary Endpoint

**GET /api/analytics/summary**

Retrieve aggregated analytics data:

```json
{
    "total_views": 10250,
    "unique_visitors": 3450,
    "avg_session_duration": 420,
    "bounce_rate": 35.5,
    "page_views_over_time": [
        {"date": "2024-01-01", "views": 150},
        {"date": "2024-01-02", "views": 180}
    ],
    "top_pages": [
        {"page": "/getting-started.html", "views": 450},
        {"page": "/examples/basics.md", "views": 380}
    ],
    "search_queries": [
        {"query": "hello world", "count": 45},
        {"query": "storage patterns", "count": 38}
    ]
}
```

## User Feedback System

### Feedback Collection

Implement a simple feedback widget:

```javascript
class FeedbackCollector {
    constructor() {
        this.initFeedbackWidget();
    }
    
    initFeedbackWidget() {
        const widget = document.createElement('div');
        widget.id = 'feedback-widget';
        widget.innerHTML = `
            <div style="position: fixed; bottom: 20px; right: 20px; background: white; 
                        padding: 15px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.15);">
                <p style="margin: 0 0 10px; font-size: 14px;">Was this helpful?</p>
                <button id="feedback-yes" style="margin-right: 10px;">Yes</button>
                <button id="feedback-no">No</button>
            </div>
        `;
        document.body.appendChild(widget);
        
        document.getElementById('feedback-yes').addEventListener('click', () => {
            this.submitFeedback('yes');
        });
        document.getElementById('feedback-no').addEventListener('click', () => {
            this.submitFeedback('no');
        });
    }
    
    submitFeedback(rating) {
        gtag('event', 'page_feedback', {
            page: window.location.pathname,
            rating: rating,
        });
    }
}

new FeedbackCollector();
```

## Privacy Considerations

- Anonymize user data
- No personally identifiable information (PII)
- Comply with GDPR/CCPA regulations
- Implement opt-out mechanisms
- Secure data transmission (HTTPS)
- Regular data cleanup policies

## Conclusion

This analytics infrastructure provides comprehensive insights into how users interact with the Soroban Cookbook documentation. Use these insights to improve documentation quality, identify knowledge gaps, and enhance the learning experience.
