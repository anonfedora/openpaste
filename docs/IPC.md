# OpenPaste IPC Design

## Overview

OpenPaste uses multiple IPC mechanisms to enable communication between the daemon and various clients (desktop app, CLI, REST API, plugins). This document details the IPC protocols, message formats, and communication patterns.

## IPC Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Daemon                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ IPC Server   │  │ REST Server  │  │ Event Bus    │      │
│  │ (Unix/TCP)   │  │  (HTTP)      │  │  (Internal)  │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
└─────────┼──────────────────┼──────────────────┼──────────────┘
          │                  │                  │
    ┌─────▼─────┐    ┌──────▼──────┐    ┌──────▼──────┐
    │  Desktop  │    │     CLI     │    │   Plugin    │
    │   Client  │    │   Client    │    │   Runtime   │
    └───────────┘    └─────────────┘    └─────────────┘
```

## IPC Mechanisms

### 1. Unix Domain Sockets (Linux/macOS)

**Purpose:** Local IPC for desktop app and CLI

**Path:**
```
Linux: /tmp/openpaste.sock
macOS: /var/run/openpaste.sock
```

**Advantages:**
- Fast (no network stack overhead)
- Secure (filesystem permissions)
- Low latency
- No port conflicts

**Implementation:**
```rust
use tokio::net::UnixListener;

async fn start_ipc_server() -> Result<(), IpcError> {
    let listener = UnixListener::bind("/tmp/openpaste.sock")?;
    
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}
```

### 2. Named Pipes (Windows)

**Purpose:** Local IPC for desktop app and CLI

**Path:**
```
\\.\pipe\OpenPaste
```

**Advantages:**
- Native Windows IPC
- Fast
- Secure (ACL-based)

**Implementation:**
```rust
use tokio::net::windows::named_pipe::ClientOptions;

async fn start_ipc_server() -> Result<(), IpcError> {
    let server = ServerOptions::new()
        .create(r"\\.\pipe\OpenPaste")?;
    
    loop {
        let stream = server.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}
```

### 3. TCP Sockets (Fallback)

**Purpose:** Cross-platform IPC fallback

**Port:** 7891 (localhost only)

**Advantages:**
- Works on all platforms
- Simple implementation
- Firewall-friendly (localhost only)

**Implementation:**
```rust
use tokio::net::TcpListener;

async fn start_tcp_server() -> Result<(), IpcError> {
    let listener = TcpListener::bind("127.0.0.1:7891").await?;
    
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}
```

### 4. REST API

**Purpose:** External integration and web clients

**Port:** 7890 (localhost only by default)

**Protocol:** HTTP/1.1 with JSON

**Advantages:**
- Language-agnostic
- Easy to use
- Standard protocol

**Details:** See [REST_API.md](REST_API.md)

### 5. WebSocket

**Purpose:** Real-time updates to clients

**Endpoint:** `ws://localhost:7890/ws`

**Protocol:** WebSocket with JSON messages

**Advantages:**
- Bidirectional
- Real-time
- Efficient for updates

## Message Protocol

### Message Format

**Serialization:** Bincode (binary) for IPC, JSON for REST

**Message Structure:**
```rust
#[derive(Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: u64,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    // Clipboard operations
    GetClipboard,
    SetClipboard,
    ClipboardChanged,
    
    // Search operations
    Search,
    GetItem,
    
    // Storage operations
    PinItem,
    FavoriteItem,
    DeleteItem,
    
    // Encryption operations
    Unlock,
    Lock,
    
    // System operations
    Ping,
    Pong,
    Status,
    
    // Error
    Error,
}
```

### Request/Response Pattern

**Request:**
```rust
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}
```

**Response:**
```rust
#[derive(Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<Error>,
}
```

**Error:**
```rust
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

## IPC Operations

### Clipboard Operations

#### Get Clipboard

**Request:**
```json
{
  "id": 1,
  "method": "clipboard.get",
  "params": {}
}
```

**Response:**
```json
{
  "id": 1,
  "result": {
    "id": 123,
    "content_type": "text",
    "content": "Hello, World!",
    "created_at": 1704067200000
  },
  "error": null
}
```

#### Set Clipboard

**Request:**
```json
{
  "id": 2,
  "method": "clipboard.set",
  "params": {
    "content": "New content",
    "content_type": "text"
  }
}
```

**Response:**
```json
{
  "id": 2,
  "result": {
    "id": 124
  },
  "error": null
}
```

### Search Operations

#### Search

**Request:**
```json
{
  "id": 3,
  "method": "search",
  "params": {
    "query": "hello",
    "limit": 50,
    "offset": 0
  }
}
```

**Response:**
```json
{
  "id": 3,
  "result": {
    "total": 10,
    "results": [
      {
        "id": 123,
        "content_preview": "Hello, World!",
        "score": 0.95
      }
    ]
  },
  "error": null
}
```

#### Get Item

**Request:**
```json
{
  "id": 4,
  "method": "item.get",
  "params": {
    "id": 123
  }
}
```

**Response:**
```json
{
  "id": 4,
  "result": {
    "id": 123,
    "content_type": "text",
    "content": "Hello, World!",
    "created_at": 1704067200000,
    "source_app": "Chrome"
  },
  "error": null
}
```

### Storage Operations

#### Pin Item

**Request:**
```json
{
  "id": 5,
  "method": "item.pin",
  "params": {
    "id": 123,
    "pinned": true
  }
}
```

**Response:**
```json
{
  "id": 5,
  "result": {
    "success": true
  },
  "error": null
}
```

#### Favorite Item

**Request:**
```json
{
  "id": 6,
  "method": "item.favorite",
  "params": {
    "id": 123,
    "favorite": true
  }
}
```

**Response:**
```json
{
  "id": 6,
  "result": {
    "success": true
  },
  "error": null
}
```

#### Delete Item

**Request:**
```json
{
  "id": 7,
  "method": "item.delete",
  "params": {
    "id": 123,
    "hard_delete": false
  }
}
```

**Response:**
```json
{
  "id": 7,
  "result": {
    "success": true
  },
  "error": null
}
```

### Encryption Operations

#### Unlock

**Request:**
```json
{
  "id": 8,
  "method": "encryption.unlock",
  "params": {
    "password": "user_password"
  }
}
```

**Response:**
```json
{
  "id": 8,
  "result": {
    "success": true
  },
  "error": null
}
```

#### Lock

**Request:**
```json
{
  "id": 9,
  "method": "encryption.lock",
  "params": {}
}
```

**Response:**
```json
{
  "id": 9,
  "result": {
    "success": true
  },
  "error": null
}
```

### System Operations

#### Ping

**Request:**
```json
{
  "id": 10,
  "method": "ping",
  "params": {}
}
```

**Response:**
```json
{
  "id": 10,
  "result": {
    "pong": true,
    "timestamp": 1704067200000
  },
  "error": null
}
```

#### Status

**Request:**
```json
{
  "id": 11,
  "method": "status",
  "params": {}
}
```

**Response:**
```json
{
  "id": 11,
  "result": {
    "version": "0.1.0",
    "status": "running",
    "clipboard_watching": true,
    "encryption_enabled": true,
    "encryption_locked": false,
    "item_count": 1234
  },
  "error": null
}
```

## WebSocket Events

### Event Format

**Message:**
```json
{
  "event": "clipboard.added",
  "data": {
    "id": 123,
    "content_type": "text",
    "content_preview": "Hello, World!"
  },
  "timestamp": 1704067200000
}
```

### Event Types

#### clipboard.added

**Triggered:** When new clipboard item is added

**Data:**
```json
{
  "id": 123,
  "content_type": "text",
  "content_preview": "Hello, World!",
  "created_at": 1704067200000
}
```

#### clipboard.changed

**Triggered:** When clipboard content changes

**Data:**
```json
{
  "hash": "abc123",
  "timestamp": 1704067200000
}
```

#### item.pinned

**Triggered:** When item is pinned/unpinned

**Data:**
```json
{
  "id": 123,
  "pinned": true
}
```

#### item.deleted

**Triggered:** When item is deleted

**Data:**
```json
{
  "id": 123,
  "hard_delete": false
}
```

#### encryption.locked

**Triggered:** When vault is locked

**Data:**
```json
{
  "reason": "inactivity"
}
```

#### encryption.unlocked

**Triggered:** When vault is unlocked

**Data:**
```json
{}
```

## Authentication

### IPC Authentication

**Strategy:** Unix filesystem permissions

**Implementation:**
- Set socket file permissions to 600 (user read/write only)
- Daemon runs as user
- Clients run as same user

**Windows:**
- Use named pipe security descriptor
- Grant access to current user only

### REST API Authentication

**Strategy:** API token or session token

**Implementation:**
```http
Authorization: Bearer <token>
```

**Token Generation:**
- Random 256-bit token
- Stored in settings
- Valid until revoked

## Connection Management

### Connection Lifecycle

**Connect:**
1. Client connects to IPC endpoint
2. Server accepts connection
3. Client sends handshake
4. Server responds with welcome

**Handshake:**
```json
{
  "type": "handshake",
  "version": "0.1.0",
  "client_type": "desktop"
}
```

**Welcome:**
```json
{
  "type": "welcome",
  "server_version": "0.1.0",
  "session_id": "abc123"
}
```

**Disconnect:**
- Client sends goodbye
- Server closes connection
- Resources cleaned up

### Keep-Alive

**Mechanism:** Ping/Pong

**Interval:** 30 seconds

**Timeout:** 60 seconds

**Implementation:**
```rust
async fn keep_alive_task(tx: Sender<Message>) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        tx.send(Message::Ping).await?;
    }
}
```

### Reconnection

**Strategy:** Exponential backoff

**Implementation:**
```rust
async fn reconnect_task() {
    let mut delay = Duration::from_secs(1);
    
    loop {
        match connect().await {
            Ok(conn) => return conn,
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(30));
            }
        }
    }
}
```

## Error Handling

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse Error | Invalid JSON |
| -32600 | Invalid Request | Invalid request object |
| -32601 | Method Not Found | Method does not exist |
| -32602 | Invalid Params | Invalid method parameters |
| -32603 | Internal Error | Internal server error |
| -32000 | Not Authenticated | Authentication required |
| -32001 | Locked | Vault is locked |
| -32002 | Not Found | Item not found |
| -32003 | Permission Denied | Insufficient permissions |

### Error Response

**Format:**
```json
{
  "id": 1,
  "result": null,
  "error": {
    "code": -32001,
    "message": "Vault is locked",
    "data": null
  }
}
```

## Performance

### Performance Targets

- **IPC Latency:** < 5ms for local operations
- **Message Throughput:** 10,000 messages/second
- **Connection Setup:** < 10ms
- **WebSocket Latency:** < 10ms for event delivery

### Optimization

**Binary Serialization:** Use bincode for IPC (faster than JSON)

**Connection Pooling:** Reuse connections

**Batching:** Batch multiple operations

**Compression:** Compress large payloads

## Security

### IPC Security

**Filesystem Permissions:**
- Socket file: 600 (user only)
- Directory: 700 (user only)

**Network Isolation:**
- Bind to localhost only
- No external network access

**Input Validation:**
- Validate all inputs
- Sanitize user input
- Limit message size

### REST API Security

**TLS:** Optional (localhost only by default)

**Rate Limiting:** 100 requests/second per client

**Input Validation:** Same as IPC

## Testing

### Unit Tests

**Test Message Serialization:**
```rust
#[test]
fn test_serialize_message() {
    let msg = IpcMessage {
        id: 1,
        message_type: MessageType::Ping,
        payload: vec![],
        timestamp: 1704067200000,
    };
    
    let serialized = bincode::serialize(&msg).unwrap();
    let deserialized: IpcMessage = bincode::deserialize(&serialized).unwrap();
    
    assert_eq!(msg.id, deserialized.id);
}
```

### Integration Tests

**Test IPC Connection:**
```rust
#[tokio::test]
async fn test_ipc_connection() {
    let server = start_test_server().await;
    let client = connect_to_ipc().await;
    
    let response = client.send_request(Request::ping()).await;
    assert!(response.is_ok());
}
```

### Load Tests

**Test Throughput:**
- Send many requests rapidly
- Measure latency
- Verify no dropped messages

## IPC API

### Client API

**Connect:**
```rust
pub async fn connect() -> Result<IpcClient, IpcError> {
    let stream = connect_to_socket().await?;
    Ok(IpcClient::new(stream))
}
```

**Send Request:**
```rust
impl IpcClient {
    pub async fn send_request(&mut self, request: Request) -> Result<Response, IpcError> {
        let serialized = bincode::serialize(&request)?;
        self.stream.write_all(&serialized).await?;
        
        let response = self.read_response().await?;
        Ok(response)
    }
}
```

**Subscribe to Events:**
```rust
impl IpcClient {
    pub async fn subscribe_events(&mut self) -> Result<Receiver<Event>, IpcError> {
        let (tx, rx) = channel(100);
        self.event_sender = Some(tx);
        Ok(rx)
    }
}
```

### Server API

**Start Server:**
```rust
pub async fn start_ipc_server() -> Result<(), IpcError> {
    let listener = create_listener()?;
    
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_client(stream));
    }
}
```

**Handle Client:**
```rust
async fn handle_client(stream: UnixStream) {
    let mut client = ClientConnection::new(stream);
    
    loop {
        match client.read_request().await {
            Ok(request) => {
                let response = handle_request(request).await;
                client.send_response(response).await;
            }
            Err(_) => break,
        }
    }
}
```

## Configuration

### IPC Settings

**Configurable:**
```json
{
  "ipc": {
    "enabled": true,
    "socket_path": "/tmp/openpaste.sock",
    "tcp_port": 7891,
    "tcp_enabled": false,
    "max_connections": 10,
    "keep_alive_interval": 30,
    "keep_alive_timeout": 60
  }
}
```

### WebSocket Settings

**Configurable:**
```json
{
  "websocket": {
    "enabled": true,
    "port": 7890,
    "path": "/ws",
    "max_connections": 10
  }
}
```

## Platform-Specific Considerations

### Linux

**Socket Path:** `/tmp/openpaste.sock`

**Permissions:** Set to 600

**SELinux:** May need policy adjustment

### macOS

**Socket Path:** `/var/run/openpaste.sock`

**Permissions:** Set to 600

**Sandbox:** May need entitlement

### Windows

**Named Pipe:** `\\.\pipe\OpenPaste`

**Security Descriptor:** Grant current user only

**Firewall:** Not needed (localhost only)

## Future Enhancements

### gRPC

**Purpose:** More efficient IPC

**Benefits:**
- Streaming
- Better performance
- Code generation

### QUIC

**Purpose:** Modern transport protocol

**Benefits:**
- UDP-based
- Built-in encryption
- Connection migration

### Shared Memory

**Purpose:** Zero-copy for large data

**Benefits:**
- Faster for large data
- Lower CPU usage
