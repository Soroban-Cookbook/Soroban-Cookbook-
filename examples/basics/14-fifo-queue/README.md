# FIFO Queue

A simple First-In-First-Out (FIFO) queue implementation for ordering tasks or events on-chain.

## What It Demonstrates

- `enqueue(item)` - Add an item to the end of the queue
- `dequeue()` - Remove and return the item at the front of the queue
- `peek()` - View the front item without removing it
- `size()` - Get the current number of items in the queue
- Head/tail index tracking for efficient queue operations
- Empty and overflow scenario handling

## Use Cases

- Task queuing and processing
- Event ordering
- Fair resource allocation
- Work queue patterns

## Key Concepts

The queue uses two indices (`head` and `tail`) stored in contract storage:
- `head`: points to the next item to dequeue
- `tail`: points to where the next item will be enqueued
- Items are stored with indices as keys
- Empty queue: `head == tail`
- Size: `tail - head`
