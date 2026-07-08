# OpenPaste Encryption Design

## Overview

OpenPaste provides optional encryption for clipboard data using AES-256-GCM with Argon2id key derivation. Encryption is designed to be secure by default, with zero-knowledge architecture where even the application cannot access data without the master password.

## Encryption Architecture

```
Master Password (user-provided)
    │
    ▼
Argon2id Key Derivation
    │
    ├── Database Key (256-bit)
    │   │
    │   ├── AES-256-GCM Encryption
    │   │
    │   └── Encrypted Clipboard Data
    │
    └── Key Hash (for verification)
        │
        └── Stored in Settings (hashed)
```

## Threat Model

### Attackers We Protect Against

**Physical Access:**
- Stolen laptop with encrypted database
- Forensic analysis of storage

**Software Attacks:**
- Malware reading database files
- Memory dumps
- Process inspection

**Insider Threats:**
- Developers with access to source code
- Cloud providers (if sync enabled)

### What We Don't Protect Against

**Keylogger:** If master password is keylogged, encryption is bypassed

**Memory Dump at Runtime:** If attacker dumps memory while unlocked, keys may be exposed

**Compromised OS:** If OS is compromised, all bets are off

## Key Derivation

### Algorithm: Argon2id

**Why Argon2id?**
- Winner of Password Hashing Competition (2015)
- Resistant to GPU/ASIC attacks
- Memory-hard, resistant to brute force
- Combines Argon2i (side-channel resistant) and Argon2d (GPU-resistant)

### Parameters

**Recommended Settings:**
```rust
const ARGON2_TIME_COST: u32 = 3;        // Number of iterations
const ARGON2_MEMORY_COST: u32 = 65536;  // 64 MB memory
const ARGON2_PARALLELISM: u32 = 4;      // Number of threads
const ARGON2_OUTPUT_LEN: usize = 32;    // 256-bit output
```

**Performance:** ~100ms on modern hardware

### Implementation

```rust
use argon2::{Argon2, Algorithm, Version, Params, PasswordHasher, PasswordHash};
use argon2::password_hash::{SaltString, rand_core::OsRng};

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], EncryptionError> {
    let params = Params::new(65536, 3, 4, Some(32))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key = [0u8; 32];
    argon2.hash_password_into(
        master_password.as_bytes(),
        salt,
        &mut key,
    )?;
    
    Ok(key)
}
```

### Salt Management

**Salt Generation:**
- Cryptographically secure random (OsRng)
- 16 bytes (128 bits)
- Unique per installation
- Stored in settings

**Salt Storage:**
```json
{
  "encryption": {
    "salt": "base64_encoded_salt",
    "key_hash": "argon2id_hash_of_master_password"
  }
}
```

## Encryption Algorithm

### AES-256-GCM

**Why AES-256-GCM?**
- NIST-approved standard
- 256-bit key (strong security)
- Authenticated encryption (AEAD)
- Built-in integrity check
- Hardware acceleration on modern CPUs

### Encryption Parameters

**Key Size:** 256 bits (32 bytes)

**Nonce Size:** 96 bits (12 bytes)

**Tag Size:** 128 bits (16 bytes)

### Implementation

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, Payload};

fn encrypt_data(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), EncryptionError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    // Generate random nonce
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    
    Ok((ciphertext, nonce_bytes))
}

fn decrypt_data(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, EncryptionError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    
    Ok(plaintext)
}
```

### Nonce Management

**Nonce Requirements:**
- Never reuse nonce with same key
- Random nonce acceptable for our use case
- 96-bit nonce provides sufficient space

**Nonce Storage:**
- Stored with encrypted data
- 12 bytes per item
- Included in database `encryption_nonce` column

### Authentication Tag

**Purpose:** Detect tampering

**Storage:** Included in ciphertext (AES-GCM format)

**Verification:** Automatic on decryption

## Key Management

### Master Password

**Requirements:**
- Minimum 8 characters
- Recommended 12+ characters
- Can include any Unicode characters
- Stored nowhere (user must remember)

**Password Policy:**
```json
{
  "encryption": {
    "min_password_length": 8,
    "require_special_char": false,
    "require_number": false,
    "require_uppercase": false
  }
}
```

### Database Key

**Derivation:**
- Derived from master password
- 256-bit (32 bytes)
- Used for all database encryption

**Storage:**
- Never stored in plaintext
- Derived on unlock
- Kept in memory while unlocked
- Zeroed on lock

### Key Hash

**Purpose:**
- Verify master password without storing it
- Detect incorrect password attempts

**Algorithm:** Argon2id (same as key derivation)

**Storage:** In settings table

**Implementation:**
```rust
fn verify_password(master_password: &str, stored_hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(stored_hash).unwrap();
    Argon2::default().verify_password(master_password.as_bytes(), &parsed_hash).is_ok()
}
```

## Memory Protection

### Secure Memory Handling

**Zeroing Sensitive Data:**
```rust
use zeroize::Zeroize;

struct SecureKey {
    key: [u8; 32],
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}
```

**Key Storage:**
- Keys wrapped in `Zeroize` types
- Automatically zeroed on drop
- Manual zeroing before explicit deletion

### Memory Locking

**mlock:** (Optional, Linux/macOS)
- Prevent swapping to disk
- Requires elevated privileges
- May not be available on all platforms

**Implementation:**
```rust
#[cfg(target_os = "linux")]
use libc::{mlock, munlock};

fn lock_memory(data: &mut [u8]) -> Result<(), EncryptionError> {
    #[cfg(target_os = "linux")]
    unsafe {
        if mlock(data.as_mut_ptr() as *mut libc::c_void, data.len()) != 0 {
            return Err(EncryptionError::MemoryLockFailed);
        }
    }
    Ok(())
}
```

### Heap Allocation

**Strategy:** Use stack allocation where possible

**Large Data:** Use heap with zeroing

**Implementation:**
```rust
// Stack allocation for keys (preferred)
let mut key = [0u8; 32];

// Heap allocation for large data
let mut data = vec![0u8; size];
data.zeroize(); // When done
```

## Locking Mechanism

### Auto-Lock

**Triggers:**
- User inactivity (configurable, default 5 minutes)
- Screen lock/sleep
- Manual lock
- Error condition

**Implementation:**
```rust
struct LockManager {
    last_activity: Arc<Mutex<Instant>>,
    auto_lock_duration: Duration,
}

impl LockManager {
    fn check_auto_lock(&self) -> bool {
        let last_activity = *self.last_activity.lock().unwrap();
        last_activity.elapsed() > self.auto_lock_duration
    }
}
```

### Manual Lock

**User Action:**
- Menu option "Lock OpenPaste"
- Keyboard shortcut (Ctrl+Shift+L)
- Tray icon option

**Behavior:**
- Zero all keys from memory
- Close database connections
- Require master password to unlock

### Unlock

**Process:**
1. User enters master password
2. Verify against stored hash
3. Derive database key
4. Open database connections
5. Keep key in memory (protected)

**Failed Attempts:**
- Rate limiting after 3 failed attempts
- Exponential backoff
- Optional: Wipe data after N failed attempts

## Encryption Scope

### What Is Encrypted

**Encrypted by Default:**
- Clipboard content (text, images, HTML, etc.)
- File paths (optional)
- Metadata (optional)

**Not Encrypted:**
- Database schema
- Indexes
- Timestamps
- Content type
- Source app name
- Collection names
- Tag names

### Configurable Encryption

**User Choice:**
```json
{
  "encryption": {
    "enabled": true,
    "encrypt_content": true,
    "encrypt_metadata": false,
    "encrypt_file_paths": false
  }
}
```

### Encryption by Type

| Data Type | Default Encrypted | Configurable |
|-----------|-------------------|--------------|
| Text content | Yes | Yes |
| Image content | Yes | Yes |
| HTML content | Yes | Yes |
| File paths | No | Yes |
| Metadata | No | Yes |
| Thumbnails | No | Yes |

## Key Rotation

### Rotation Strategy

**Trigger:**
- User-initiated
- Time-based (optional, e.g., yearly)
- Security event

**Process:**
1. User provides new master password
2. Derive new key
3. Re-encrypt all data with new key
4. Update key hash
5. Delete old key

### Implementation

```rust
async fn rotate_keys(old_password: &str, new_password: &str) -> Result<(), EncryptionError> {
    // Derive old key
    let old_key = derive_key(old_password, &salt)?;
    
    // Derive new key
    let new_key = derive_key(new_password, &new_salt)?;
    
    // Get all encrypted items
    let items = db::get_all_encrypted_items().await?;
    
    // Re-encrypt each item
    for item in items {
        let decrypted = decrypt_data(&old_key, &item.content, &item.nonce)?;
        let (encrypted, new_nonce) = encrypt_data(&new_key, &decrypted)?;
        db::update_encrypted_item(item.id, &encrypted, &new_nonce).await?;
    }
    
    // Update key hash
    update_key_hash(new_password)?;
    
    Ok(())
}
```

## Database Encryption

### Encryption at Rest

**Strategy:** Encrypt content column

**Implementation:**
```sql
-- Content is encrypted before storage
UPDATE clipboard_items
SET content = ?,
    encryption_nonce = ?,
    is_encrypted = 1
WHERE id = ?;
```

### SQLCipher Alternative

**Consideration:** SQLCipher provides transparent encryption

**Decision:** Not using SQLCipher because:
- Want explicit control over encryption
- Need custom key derivation
- Want to encrypt only specific columns
- SQLCipher adds dependency

### WAL Mode with Encryption

**Challenge:** WAL file also needs protection

**Solution:**
- Encrypt content before writing to WAL
- WAL contains encrypted data
- No special handling needed

## File System Encryption

### Encrypted Files

**Strategy:** Encrypt file contents before writing

**Implementation:**
```rust
fn store_encrypted_file(key: &[u8; 32], data: &[u8], path: &Path) -> Result<(), EncryptionError> {
    let (encrypted, nonce) = encrypt_data(key, data)?;
    
    // Store nonce in file header
    let mut file = File::create(path)?;
    file.write_all(&nonce)?;
    file.write_all(&encrypted)?;
    
    Ok(())
}
```

### File Naming

**Strategy:** Keep original filename, encrypt content

**Alternative:** Encrypt filename (adds complexity)

**Decision:** Encrypt content only, keep filename readable

## Performance

### Encryption Performance

**Targets:**
- Key derivation: < 100ms
- Encryption (1MB): < 50ms
- Decryption (1MB): < 50ms

### Optimization

**Hardware Acceleration:**
- AES-NI on x86
- ARM Crypto Extensions on ARM
- Automatic with RustCrypto

**Batch Operations:**
- Reuse cipher context
- Batch encrypt/decrypt
- Parallel processing where safe

**Caching:**
- Cache derived key while unlocked
- Avoid repeated key derivation

## Error Handling

### Encryption Errors

**Types:**
- Key derivation failure
- Encryption failure
- Decryption failure (wrong key, corrupted data)

**Handling:**
- Log error (without sensitive data)
- Notify user
- Suggest recovery options

### Decryption Errors

**Wrong Password:**
- Clear error message
- Increment failed attempt counter
- Rate limit after attempts

**Corrupted Data:**
- Log corruption
- Attempt recovery from backup
- Notify user of data loss

## Backup and Recovery

### Encrypted Backup

**Strategy:** Backup encrypted data

**Process:**
1. Export encrypted database
2. Export encryption salt
3. Export key hash (for password verification)
4. User must remember master password

**Backup Format:**
```
openpaste-backup-2024-01-01/
├── openpaste.db (encrypted)
├── encryption-salt.bin
└── key-hash.txt
```

### Recovery

**From Backup:**
1. Restore database file
2. Restore encryption salt
3. Restore key hash
4. Unlock with master password

**Lost Master Password:**
- Data is unrecoverable (by design)
- This is intentional for security
- Warn user during setup

## Security Best Practices

### Password Security

**Recommendations to Users:**
- Use unique password for OpenPaste
- Use password manager
- Don't share password
- Don't write password down
- Use passphrase if easier to remember

### Key Storage

**Never Store:**
- Master password
- Database key (plaintext)
- Encryption keys in logs

**Always Store:**
- Key hash (for verification)
- Encryption salt
- Nonces (with encrypted data)

### Memory Safety

**Always:**
- Zero sensitive data after use
- Use secure memory types
- Lock memory if possible
- Minimize time keys are in memory

**Never:**
- Log sensitive data
- Print keys to console
- Leave keys in memory longer than needed
- Swap keys to disk

## Encryption API

### Initialize Encryption

```rust
pub async fn initialize_encryption(master_password: &str) -> Result<EncryptionState, EncryptionError> {
    // Generate salt
    let salt = generate_salt();
    
    // Derive key
    let key = derive_key(master_password, &salt)?;
    
    // Hash password for verification
    let key_hash = hash_password(master_password, &salt)?;
    
    // Store salt and hash
    store_encryption_metadata(&salt, &key_hash).await?;
    
    Ok(EncryptionState { key, salt })
}
```

### Unlock

```rust
pub async fn unlock(master_password: &str) -> Result<EncryptionState, EncryptionError> {
    // Retrieve salt and stored hash
    let (salt, stored_hash) = retrieve_encryption_metadata().await?;
    
    // Verify password
    if !verify_password(master_password, &stored_hash) {
        return Err(EncryptionError::InvalidPassword);
    }
    
    // Derive key
    let key = derive_key(master_password, &salt)?;
    
    Ok(EncryptionState { key, salt })
}
```

### Lock

```rust
pub fn lock(state: EncryptionState) {
    // Key is zeroed when state is dropped
    drop(state);
}
```

### Encrypt Item

```rust
pub fn encrypt_item(state: &EncryptionState, data: &[u8]) -> Result<EncryptedData, EncryptionError> {
    let (ciphertext, nonce) = encrypt_data(&state.key, data)?;
    Ok(EncryptedData { ciphertext, nonce })
}
```

### Decrypt Item

```rust
pub fn decrypt_item(state: &EncryptionState, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
    decrypt_data(&state.key, &encrypted.ciphertext, &encrypted.nonce)
}
```

## Testing

### Unit Tests

- Key derivation
- Encryption/decryption
- Memory zeroing
- Password verification
- Nonce uniqueness

### Integration Tests

- End-to-end encryption workflow
- Database encryption
- File system encryption
- Lock/unlock cycle

### Security Tests

- Known-answer tests (KAT)
- Side-channel resistance
- Memory leak detection
- Key exposure detection

## Compliance

### Standards

**NIST:**
- AES-256-GCM (FIPS 197)
- Argon2id (Password Hashing Competition winner)

**OWASP:**
- Password storage best practices
- Key management guidelines

### Regulations

**GDPR:**
- Encryption at rest
- Data protection by design
- Right to be forgotten (deletion)

**HIPAA:**
- (If used in healthcare context)
- Encryption requirements
- Access controls

## Future Enhancements

### Hardware Security Module (HSM)

**Purpose:** Store keys in hardware

**Benefits:**
- Keys never in software memory
- Tamper resistance
- FIPS 140-2 compliance

### Biometric Unlock

**Platforms:**
- Windows Hello
- Touch ID (macOS)
- Linux fingerprint readers

**Implementation:**
- Biometric unlocks encryption key
- Key still protected by master password
- Fallback to password if biometric fails

### Multi-Factor Authentication

**Optional:** Require second factor to unlock

**Factors:**
- Hardware token (YubiKey)
- TOTP app
- SMS (less secure)

### Secure Enclave

**Platforms:**
- Apple Secure Enclave
- Windows TPM
- Linux TPM

**Benefits:**
- Hardware-protected keys
- No software access to keys
