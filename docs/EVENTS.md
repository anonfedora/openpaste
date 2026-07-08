# OpenPaste Event System

## Overview

OpenPaste uses an internal event bus to enable decoupled communication between components. The event system follows the publish-subscribe pattern, allowing components to react to clipboard changes, search operations, and other system events without tight coupling.

## Event Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Event Bus                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Subscribers │  │  Publishers │  │  Channels   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Storage    │    │   Search     │    │   Plugin     │
│   Engine     │    │   Engine     │    │   Runtime    │
└──────────────┘    └──────────────┘    └──────────────┘
```

## Event Types

### Clipboard Events

#### ClipboardAdded

**Triggered:** When new clipboard item is captured

**Payload:**
```rust
pub struct ClipboardAddedEvent {
    pub item_id: i64,
    pub content_type: ContentType,
    pub hash: String,
    pub source_app: Option<String>,
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (store item)
- Search Engine (update index)
- Plugin Runtime (notify plugins)
- Desktop UI (update UI)
- Sync Engine (queue for sync)

#### ClipboardChanged

**Triggered:** When clipboard content changes (not captured)

**Payload:**
```rust
pub struct ClipboardChangedEvent {
    pub previous_hash: Option<String>,
    pub new_hash: String,
    pub timestamp: i64,
}
```

**Subscribers:**
- Clipboard Watcher (trigger capture)
- Desktop UI (show notification)

#### ClipboardAccessed

**Triggered:** When clipboard item is accessed

**Payload:**
```rust
pub struct ClipboardAccessedEvent {
    pub item_id: i64,
    pub source: AccessSource,
    pub timestamp: i64,
}

pub enum AccessSource {
    Desktop,
    Cli,
    Plugin,
    Api,
}
```

**Subscribers:**
- Storage Engine (update access count)
- Audit Log (log access)
- Analytics (track usage)

#### ClipboardPinned

**Triggered:** When item is pinned/unpinned

**Payload:**
```rust
pub struct ClipboardPinnedEvent {
    pub item_id: i64,
    pub pinned: bool,
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (update database)
- Desktop UI (update UI)

#### ClipboardDeleted

**Triggered:** When item is deleted

**Payload:**
```rust
pub struct ClipboardDeletedEvent {
    pub item_id: i64,
    pub hard_delete: bool,
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (delete from database)
- Search Engine (remove from index)
- Plugin Runtime (notify plugins)
- Desktop UI (update UI)
- Sync Engine (queue deletion)

### Search Events

#### SearchPerformed

**Triggered:** When search is executed

**Payload:**
```rust
pub struct SearchPerformedEvent {
    pub query: String,
    pub result_count: usize,
    pub latency_ms: u64,
    pub timestamp: i64,
}
```

**Subscribers:**
- Analytics (track searches)
- Plugin Runtime (notify plugins)

#### SearchResultSelected

**Triggered:** When search result is selected

**Payload:**
```rust
pub struct SearchResultSelectedEvent {
    pub item_id: i64,
    pub query: String,
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (update access count)
- Analytics (track result selection)

### Storage Events

#### StorageThresholdReached

**Triggered:** When storage limit is approached

**Payload:**
```rust
pub struct StorageThresholdReachedEvent {
    pub current_size: u64,
    pub max_size: u64,
    pub threshold_percent: f32,
}
```

**Subscribers:**
- Storage Engine (trigger cleanup)
- Desktop UI (show warning)

#### ItemRetentionTriggered

**Triggered:** When retention policy triggers cleanup

**Payload:**
```rust
pub struct ItemRetentionTriggeredEvent {
    pub items_to_delete: Vec<i64>,
    pub reason: RetentionReason,
}

pub enum RetentionReason {
    AgeExceeded,
    CountExceeded,
    Manual,
}
```

**Subscribers:**
- Storage Engine (delete items)
- Desktop UI (show notification)

### Encryption Events

#### EncryptionStateChanged

**Triggered:** When encryption is enabled/disabled

**Payload:**
```rust
pub struct EncryptionStateChangedEvent {
    pub enabled: bool,
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (encrypt/decrypt data)
- Desktop UI (update UI)
- Plugin Runtime (notify plugins)

#### VaultLocked

**Triggered:** When vault is locked

**Payload:**
```rust
pub struct VaultLockedEvent {
    pub reason: LockReason,
    pub timestamp: i64,
}

pub enum LockReason {
    Manual,
    Inactivity,
    Sleep,
    Error,
}
```

**Subscribers:**
- Storage Engine (zero keys)
- Desktop UI (show lock screen)
- Plugin Runtime (pause plugins)

#### VaultUnlocked

**Triggered:** When vault is unlocked

**Payload:**
```rust
pub struct VaultUnlockedEvent {
    pub timestamp: i64,
}
```

**Subscribers:**
- Storage Engine (derive keys)
- Desktop UI (hide lock screen)
- Plugin Runtime (resume plugins)

### Sync Events

#### SyncStarted

**Triggered:** When sync operation starts

**Payload:**
```rust
pub struct SyncStartedEvent {
    pub provider: String,
    pub timestamp: i64,
}
```

**Subscribers:**
- Desktop UI (show sync indicator)
- Analytics (track sync)

#### SyncCompleted

**Triggered:** When sync operation completes

**Payload:**
```rust
pub struct SyncCompletedEvent {
    pub provider: String,
    pub items_synced: usize,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

**Subscribers:**
- Desktop UI (hide sync indicator)
- Analytics (track sync results)

#### SyncConflictDetected

**Triggered:** When sync conflict is detected

**Payload:**
```rust
pub struct SyncConflictDetectedEvent {
    pub item_id: i64,
    pub local_version: i64,
    pub remote_version: i64,
    pub timestamp: i64,
}
```

**Subscribers:**
- Sync Engine (resolve conflict)
- Desktop UI (show conflict dialog)

### Plugin Events

#### PluginLoaded

**Triggered:** When plugin is loaded

**Payload:**
```rust
pub struct PluginLoadedEvent {
    pub plugin_name: String,
    pub plugin_version: String,
    pub timestamp: i64,
}
```

**Subscribers:**
- Plugin Runtime (initialize plugin)
- Desktop UI (update plugin list)

#### PluginUnloaded

**Triggered:** When plugin is unloaded

**Payload:**
```rust
pub struct PluginUnloadedEvent {
    pub plugin_name: String,
    pub reason: UnloadReason,
}

pub enum UnloadReason {
    User,
    Error,
    Shutdown,
}
```

**Subscribers:**
- Plugin Runtime (cleanup plugin)
- Desktop UI (update plugin list)

#### PluginError

**Triggered:** When plugin encounters error

**Payload:**
```rust
pub struct PluginErrorEvent {
    pub plugin_name: String,
    pub error: String,
    pub severity: ErrorSeverity,
}

pub enum ErrorSeverity {
    Warning,
    Error,
    Fatal,
}
```

**Subscribers:**
- Plugin Runtime (handle error)
- Desktop UI (show error notification)
- Analytics (track plugin errors)

### System Events

#### DaemonStarted

**Triggered:** When daemon starts

**Payload:**
```rust
pub struct DaemonStartedEvent {
    pub version: String,
    pub timestamp: i64,
}
```

**Subscribers:**
- All components (initialize)
- Desktop UI (connect)

#### DaemonShutdown

**Triggered:** When daemon is shutting down

**Payload:**
```rust
pub struct DaemonShutdownEvent {
    pub reason: ShutdownReason,
}

pub enum ShutdownReason {
    User,
    Update,
    Error,
    System,
}
```

**Subscribers:**
- All components (cleanup)
- Desktop UI (disconnect)

#### ClientConnected

**Triggered:** When client connects to daemon

**Payload:**
```rust
pub struct ClientConnectedEvent {
    pub client_id: String,
    pub client_type: ClientType,
    pub timestamp: i64,
}

pub enum ClientType {
    Desktop,
    Cli,
    Api,
}
```

**Subscribers:**
- Desktop UI (show connected clients)
- Analytics (track connections)

#### ClientDisconnected

**Triggered:** When client disconnects

**Payload:**
```rust
pub struct ClientDisconnectedEvent {
    pub client_id: String,
    pub reason: DisconnectReason,
}

pub enum DisconnectReason {
    User,
    Error,
    Timeout,
}
```

**Subscribers:**
- Desktop UI (update connected clients)
- Analytics (track disconnections)

## Event Bus Implementation

### Channel-Based Architecture

**Library:** Tokio channels

**Implementation:**
```rust
use tokio::sync::broadcast;

pub struct EventBus {
    clipboard_added: broadcast::Sender<ClipboardAddedEvent>,
    clipboard_changed: broadcast::Sender<ClipboardChangedEvent>,
    // ... other channels
}

impl EventBus {
    pub fn new() -> Self {
        let (clipboard_added, _) = broadcast::channel(100);
        let (clipboard_changed, _) = broadcast::channel(100);
        
        Self {
            clipboard_added,
            clipboard_changed,
        }
    }
    
    pub fn publish_clipboard_added(&self, event: ClipboardAddedEvent) {
        let _ = self.clipboard_added.send(event);
    }
    
    pub fn subscribe_clipboard_added(&self) -> broadcast::Receiver<ClipboardAddedEvent> {
        self.clipboard_added.subscribe()
    }
}
```

### Generic Event Bus

**Alternative:** Single channel with enum

**Implementation:**
```rust
pub enum Event {
    ClipboardAdded(ClipboardAddedEvent),
    ClipboardChanged(ClipboardChangedEvent),
    // ... other events
}

pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}
```

**Decision:** Use separate channels for each event type (better performance, clearer semantics)

## Subscription Management

### Subscription Pattern

**Subscriber:**
```rust
pub struct EventSubscriber {
    clipboard_added_rx: broadcast::Receiver<ClipboardAddedEvent>,
    clipboard_changed_rx: broadcast::Receiver<ClipboardChangedEvent>,
}

impl EventSubscriber {
    pub async fn run(&mut self) {
        tokio::select! {
            event = self.clipboard_added_rx.recv() => {
                if let Ok(event) = event {
                    self.handle_clipboard_added(event).await;
                }
            }
            event = self.clipboard_changed_rx.recv() => {
                if let Ok(event) = event {
                    self.handle_clipboard_changed(event).await;
                }
            }
        }
    }
}
```

### Subscription Filtering

**Filter by Type:**
```rust
pub struct FilteredSubscriber {
    rx: broadcast::Receiver<Event>,
    filter: Box<dyn Fn(&Event) -> bool + Send>,
}

impl FilteredSubscriber {
    pub async fn recv_filtered(&mut self) -> Option<Event> {
        loop {
            if let Ok(event) = self.rx.recv().await {
                if (self.filter)(&event) {
                    return Some(event);
                }
            }
        }
    }
}
```

### Unsubscription

**Automatic:** Drop receiver to unsubscribe

**Manual:** Call `close()` on receiver

## Backpressure Management

### Channel Capacity

**Default:** 100 messages per channel

**Overflow Strategy:** Drop oldest messages

**Configuration:**
```rust
let (tx, _rx) = broadcast::channel(100); // 100 message buffer
```

### Backpressure Handling

**Slow Subscribers:**
- Use bounded channels
- Drop messages if buffer full
- Log dropped messages

**Fast Publishers:**
- Use unbounded channels (careful)
- Implement rate limiting
- Batch events

## Error Handling

### Event Publishing Errors

**Channel Full:**
- Log warning
- Drop event (or block)
- Continue operation

**No Subscribers:**
- Ignore (expected)
- No error

### Event Processing Errors

**Subscriber Error:**
- Log error
- Continue processing other events
- Don't crash event bus

**Poisoned Subscriber:**
- Remove subscriber
- Log error
- Restart subscriber if critical

## Performance Considerations

### Event Throughput

**Target:** 10,000 events/second

**Latency:** < 1ms from publish to receive

### Optimization

**Zero-Copy:** Use Arc for large payloads

**Batching:** Batch similar events

**Async:** Non-blocking event processing

**Memory Pooling:** Reuse event objects

## Event Ordering

### Ordering Guarantees

**Per-Channel:** Events are ordered within a channel

**Cross-Channel:** No ordering guarantees across channels

**Implementation:** Use sequence numbers if needed

### Sequence Numbers

**Optional:** Add sequence to events

**Implementation:**
```rust
pub struct Event {
    pub sequence: u64,
    pub payload: EventPayload,
}
```

## Event Persistence

### Event Log (Optional)

**Purpose:** Audit trail, replay capability

**Storage:** Separate table in database

**Schema:**
```sql
CREATE TABLE event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    sequence INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
```

**Replay:**
- Load events from log
- Replay in order
- Useful for debugging

## Event Testing

### Unit Tests

**Test Event Publishing:**
```rust
#[tokio::test]
async fn test_publish_clipboard_added() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe_clipboard_added();
    
    let event = ClipboardAddedEvent {
        item_id: 1,
        // ... other fields
    };
    
    bus.publish_clipboard_added(event.clone());
    
    let received = rx.recv().await.unwrap();
    assert_eq!(received.item_id, event.item_id);
}
```

### Integration Tests

**Test Event Flow:**
- Publish event
- Verify all subscribers receive
- Verify processing order

### Load Tests

**Test Throughput:**
- Publish many events rapidly
- Measure latency
- Verify no dropped events

## Event API

### Publish Event

```rust
pub async fn publish_event(event: Event) -> Result<(), EventError> {
    EVENT_BUS.publish(event).await?;
    Ok(())
}
```

### Subscribe to Events

```rust
pub async fn subscribe_to_events() -> broadcast::Receiver<Event> {
    EVENT_BUS.subscribe()
}
```

### Subscribe to Specific Event Type

```rust
pub async fn subscribe_clipboard_added() -> broadcast::Receiver<ClipboardAddedEvent> {
    EVENT_BUS.subscribe_clipboard_added()
}
```

## Event Configuration

### Channel Sizes

**Configurable:**
```json
{
  "events": {
    "channel_capacity": 100,
    "enable_event_log": false,
    "max_event_log_size": 10000
  }
}
```

### Event Filtering

**Global Filters:**
- Filter by severity
- Filter by source
- Filter by type

## Event Debugging

### Event Logging

**Development:** Log all events

**Production:** Log only errors and warnings

**Implementation:**
```rust
fn log_event(event: &Event) {
    if cfg!(debug_assertions) {
        debug!("Event: {:?}", event);
    }
}
```

### Event Tracing

**Distributed Tracing:** Add span context to events

**Implementation:**
```rust
pub struct Event {
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub payload: EventPayload,
}
```

## Event Security

### Event Validation

**Validate Payloads:**
- Check required fields
- Validate data types
- Sanitize user input

### Event Access Control

**Plugin Access:**
- Plugins can only subscribe to allowed events
- Plugins cannot publish sensitive events
- Permission-based access

## Future Enhancements

### Event Replay

**Purpose:** Debugging, testing, recovery

**Implementation:**
- Record events to log
- Replay from log
- Selective replay by type/time

### Event Aggregation

**Purpose:** Reduce event volume

**Examples:**
- Aggregate multiple ClipboardAdded events
- Aggregate search events
- Aggregate errors

### Event Routing

**Purpose:** Send events to specific subscribers

**Implementation:**
- Topic-based routing
- Pattern matching
- Conditional routing

### Remote Events

**Purpose:** Cross-process events

**Implementation:**
- Network event bus
- WebSockets
- Message queue (RabbitMQ, Kafka)
