# Analytics Backend Setup Guide

## Overview

This guide provides instructions for setting up the analytics backend infrastructure for the Soroban Cookbook documentation site.

## Prerequisites

- Node.js 14+ or Python 3.8+
- Access to a database (PostgreSQL recommended)
- Environment variables configured

## Option 1: Node.js Backend

### Installation

```bash
npm install express cors mongodb dotenv
```

### Create Analytics Server

Create `analytics-server.js`:

```javascript
const express = require('express');
const cors = require('cors');
const { MongoClient } = require('mongodb');
require('dotenv').config();

const app = express();
const uri = process.env.MONGODB_URI;
const client = new MongoClient(uri);

// Middleware
app.use(cors());
app.use(express.json());

// Connect to database
async function connect() {
    try {
        await client.connect();
        console.log('Connected to MongoDB');
    } catch (error) {
        console.error('Failed to connect to MongoDB:', error);
        process.exit(1);
    }
}

// Analytics event collection endpoint
app.post('/api/analytics', async (req, res) => {
    try {
        const db = client.db('analytics');
        const collection = db.collection('events');
        
        const event = {
            ...req.body,
            collectedAt: new Date(),
            ipAddress: req.ip,
        };
        
        const result = await collection.insertOne(event);
        res.json({ success: true, id: result.insertedId });
    } catch (error) {
        console.error('Failed to save analytics event:', error);
        res.status(500).json({ error: 'Failed to save event' });
    }
});

// Analytics summary endpoint
app.get('/api/analytics/summary', async (req, res) => {
    try {
        const db = client.db('analytics');
        const collection = db.collection('events');
        
        // Get date range (last 30 days)
        const thirtyDaysAgo = new Date();
        thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
        
        const events = await collection
            .find({ collectedAt: { $gte: thirtyDaysAgo } })
            .toArray();
        
        // Calculate metrics
        const summary = calculateSummary(events);
        
        res.json(summary);
    } catch (error) {
        console.error('Failed to get analytics summary:', error);
        res.status(500).json({ error: 'Failed to get summary' });
    }
});

// Top pages endpoint
app.get('/api/analytics/top-pages', async (req, res) => {
    try {
        const db = client.db('analytics');
        const collection = db.collection('events');
        
        const topPages = await collection
            .aggregate([
                { $match: { type: 'page_view' } },
                { $group: { _id: '$page', views: { $sum: 1 } } },
                { $sort: { views: -1 } },
                { $limit: 10 },
                { $project: { _id: 0, page: '$_id', views: 1 } },
            ])
            .toArray();
        
        res.json(topPages);
    } catch (error) {
        console.error('Failed to get top pages:', error);
        res.status(500).json({ error: 'Failed to get top pages' });
    }
});

// Search queries endpoint
app.get('/api/analytics/search-queries', async (req, res) => {
    try {
        const db = client.db('analytics');
        const collection = db.collection('events');
        
        const searches = await collection
            .aggregate([
                { $match: { type: 'search' } },
                { $group: { _id: '$query', count: { $sum: 1 } } },
                { $sort: { count: -1 } },
                { $limit: 20 },
                { $project: { _id: 0, query: '$_id', count: 1 } },
            ])
            .toArray();
        
        res.json(searches);
    } catch (error) {
        console.error('Failed to get search queries:', error);
        res.status(500).json({ error: 'Failed to get search queries' });
    }
});

// Utility function to calculate summary metrics
function calculateSummary(events) {
    const pageViews = events.filter(e => e.type === 'page_view');
    const uniqueVisitors = new Set(events.map(e => e.userId)).size;
    
    const sessionDurations = [];
    const sessionMap = {};
    
    events.forEach(event => {
        if (!sessionMap[event.sessionId]) {
            sessionMap[event.sessionId] = [];
        }
        sessionMap[event.sessionId].push(event);
    });
    
    Object.values(sessionMap).forEach(session => {
        if (session.length > 0) {
            const times = session.map(e => new Date(e.timestamp).getTime());
            const duration = (Math.max(...times) - Math.min(...times)) / 1000;
            sessionDurations.push(duration);
        }
    });
    
    const avgSessionDuration = sessionDurations.length > 0
        ? sessionDurations.reduce((a, b) => a + b) / sessionDurations.length
        : 0;
    
    const bounceCount = events.filter(e => e.type === 'page_view' && !e.timeOnPage).length;
    const bounceRate = pageViews.length > 0 ? (bounceCount / pageViews.length) * 100 : 0;
    
    return {
        total_views: pageViews.length,
        unique_visitors: uniqueVisitors,
        avg_session_duration: Math.round(avgSessionDuration),
        bounce_rate: bounceRate.toFixed(1),
        page_views_over_time: aggregateByDate(pageViews),
        top_pages: [],
        search_queries: [],
    };
}

function aggregateByDate(events) {
    const dateMap = {};
    
    events.forEach(event => {
        const date = new Date(event.collectedAt);
        const dateString = date.toISOString().split('T')[0];
        
        if (!dateMap[dateString]) {
            dateMap[dateString] = 0;
        }
        dateMap[dateString]++;
    });
    
    return Object.entries(dateMap)
        .map(([date, views]) => ({ date, views }))
        .sort((a, b) => new Date(a.date) - new Date(b.date));
}

// Start server
const PORT = process.env.PORT || 3000;
connect().then(() => {
    app.listen(PORT, () => {
        console.log(`Analytics server running on port ${PORT}`);
    });
});

// Graceful shutdown
process.on('SIGTERM', async () => {
    await client.close();
    process.exit(0);
});
```

### Environment Configuration

Create `.env`:

```env
MONGODB_URI=mongodb://localhost:27017/analytics
PORT=3000
NODE_ENV=production
```

### Run the Server

```bash
node analytics-server.js
```

## Option 2: Python Backend (Flask)

### Installation

```bash
pip install flask flask-cors pymongo python-dotenv
```

### Create Analytics Server

Create `analytics_server.py`:

```python
from flask import Flask, request, jsonify
from flask_cors import CORS
from pymongo import MongoClient
from datetime import datetime, timedelta
from os import getenv
from dotenv import load_dotenv

load_dotenv()

app = Flask(__name__)
CORS(app)

# MongoDB connection
mongodb_uri = getenv('MONGODB_URI', 'mongodb://localhost:27017')
client = MongoClient(mongodb_uri)
db = client['analytics']
events_collection = db['events']

@app.route('/api/analytics', methods=['POST'])
def collect_event():
    """Collect analytics events"""
    try:
        event = request.json
        event['collectedAt'] = datetime.utcnow()
        event['ipAddress'] = request.remote_addr
        
        result = events_collection.insert_one(event)
        return jsonify({'success': True, 'id': str(result.inserted_id)})
    except Exception as e:
        print(f'Error collecting event: {e}')
        return jsonify({'error': 'Failed to save event'}), 500

@app.route('/api/analytics/summary', methods=['GET'])
def get_summary():
    """Get analytics summary"""
    try:
        # Last 30 days
        thirty_days_ago = datetime.utcnow() - timedelta(days=30)
        
        events = list(events_collection.find({
            'collectedAt': {'$gte': thirty_days_ago}
        }))
        
        # Calculate metrics
        page_views = [e for e in events if e['type'] == 'page_view']
        unique_visitors = len(set(e['userId'] for e in events))
        
        # Session duration
        session_durations = []
        sessions = {}
        
        for event in events:
            session_id = event.get('sessionId')
            if session_id not in sessions:
                sessions[session_id] = []
            sessions[session_id].append(event)
        
        for session_events in sessions.values():
            if session_events:
                times = [e['collectedAt'].timestamp() for e in session_events]
                duration = max(times) - min(times)
                session_durations.append(duration)
        
        avg_session_duration = (
            sum(session_durations) / len(session_durations)
            if session_durations else 0
        )
        
        # Bounce rate
        bounce_rate = (
            len([e for e in page_views if e.get('timeOnPage', 0) == 0]) /
            len(page_views) * 100
            if page_views else 0
        )
        
        return jsonify({
            'total_views': len(page_views),
            'unique_visitors': unique_visitors,
            'avg_session_duration': int(avg_session_duration),
            'bounce_rate': round(bounce_rate, 1),
            'page_views_over_time': aggregate_by_date(page_views),
        })
    except Exception as e:
        print(f'Error getting summary: {e}')
        return jsonify({'error': 'Failed to get summary'}), 500

@app.route('/api/analytics/top-pages', methods=['GET'])
def get_top_pages():
    """Get top pages by views"""
    try:
        pipeline = [
            {'$match': {'type': 'page_view'}},
            {'$group': {'_id': '$page', 'views': {'$sum': 1}}},
            {'$sort': {'views': -1}},
            {'$limit': 10},
            {'$project': {'_id': 0, 'page': '$_id', 'views': 1}},
        ]
        
        result = list(events_collection.aggregate(pipeline))
        return jsonify(result)
    except Exception as e:
        print(f'Error getting top pages: {e}')
        return jsonify({'error': 'Failed to get top pages'}), 500

def aggregate_by_date(events):
    """Aggregate events by date"""
    date_map = {}
    
    for event in events:
        date_str = event['collectedAt'].strftime('%Y-%m-%d')
        date_map[date_str] = date_map.get(date_str, 0) + 1
    
    return [
        {'date': date, 'views': views}
        for date, views in sorted(date_map.items())
    ]

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=False)
```

### Run the Server

```bash
python analytics_server.py
```

## Database Schema

### MongoDB Collection: events

```javascript
{
  _id: ObjectId,
  type: "page_view" | "search" | "link_click" | "scroll_depth" | "page_engagement",
  page: "/path/to/page",
  title: "Page Title",
  timestamp: ISODate,
  sessionId: "session_xxxxx",
  userId: "user_xxxxx",
  collectedAt: ISODate,
  ipAddress: "192.168.1.1",
  
  // For page_view events
  referrer: "https://google.com",
  
  // For search events
  query: "search term",
  
  // For link_click events
  href: "/path/to/link",
  text: "Link Text",
  
  // For scroll_depth events
  depth: 25 | 50 | 75 | 100,
  
  // For page_engagement events
  timeOnPage: 120
}
```

## Deployment

### Using Docker

Create `Dockerfile`:

```dockerfile
FROM node:16-alpine

WORKDIR /app
COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 3000
CMD ["node", "analytics-server.js"]
```

Build and run:

```bash
docker build -t soroban-analytics .
docker run -e MONGODB_URI=mongodb://mongo:27017 \
           -p 3000:3000 \
           soroban-analytics
```

## Testing the Endpoints

```bash
# Send a test event
curl -X POST http://localhost:3000/api/analytics \
  -H "Content-Type: application/json" \
  -d '{
    "type": "page_view",
    "page": "/getting-started",
    "title": "Getting Started",
    "sessionId": "test_session",
    "userId": "test_user"
  }'

# Get summary
curl http://localhost:3000/api/analytics/summary
```

## Maintenance

- Set up data retention policies (e.g., keep 90 days of data)
- Regular backups of analytics database
- Monitor disk usage and database performance
- Implement log rotation for server logs

## Security

- Use HTTPS in production
- Implement authentication for sensitive endpoints
- Rate limiting on analytics collection
- CORS configuration for trusted origins
- Input validation and sanitization
