# OpenPaste Security Policy

## Overview

OpenPaste is designed with security as a core principle. This document outlines the security architecture, threat model, and security best practices for the project.

## Security Principles

1. **Defense in Depth:** Multiple layers of security
2. **Least Privilege:** Minimal access required
3. **Secure by Default:** Secure configurations out of the box
4. **Transparency:** Open security practices
5. **Regular Audits:** Continuous security review

## Threat Model

### Attackers

**Local Attacker:**
- Has physical access to device
- Can run arbitrary code
- Can access filesystem

**Network Attacker:**
- Can intercept network traffic
- Can attack network services
- Limited to localhost by default

**Malicious Plugin:**
- Runs in WASM sandbox
- Has limited permissions
- Cannot escape sandbox (by design)

### Assets to Protect

**Clipboard Data:**
- User clipboard history
- Sensitive information (passwords, keys, etc.)
- Personal information

**Encryption Keys:**
- Master password
- Database encryption keys
- Plugin permissions

**User Privacy:**
- Usage patterns
- Source applications
- Search queries

### Security Boundaries

**Process Boundary:**
- Daemon runs as user process
- No privilege escalation
- No system-level access

**Network Boundary:**
- API binds to localhost only
- No external network access by default
- Optional TLS for remote access

**Sandbox Boundary:**
- Plugins run in WASM sandbox
- No direct filesystem access
- No direct network access (without permission)

## Security Architecture

### Encryption

**At Rest:**
- AES-256-GCM for database content
- Argon2id for key derivation
- Optional file system encryption

**In Transit:**
- Optional TLS for REST API
- Unix domain sockets (local only)
- Named pipes (Windows, local only)

**In Memory:**
- Zeroization of sensitive data
- Memory locking (optional)
- Secure memory allocation

### Access Control

**Authentication:**
- API token for REST API
- Master password for encryption
- Optional biometric unlock

**Authorization:**
- Plugin permissions system
- Role-based access (future)
- Resource-based access (future)

### Isolation

**Process Isolation:**
- Separate daemon process
- Separate UI process
- IPC communication only

**Sandbox Isolation:**
- WASM runtime for plugins
- Limited host API access
- Permission-based access

## Security Features

### Encryption

**Master Password:**
- Minimum 8 characters
- Argon2id key derivation
- Salt stored separately
- Key hash for verification

**Database Encryption:**
- Per-item encryption
- Random nonces
- Authenticated encryption (AEAD)
- Optional encryption scope

**Memory Protection:**
- Zeroization on drop
- Secure memory types
- Optional memory locking

### Input Validation

**Clipboard Input:**
- Content type validation
- Size limits
- Sanitization (HTML)
- Malware scanning (future)

**API Input:**
- JSON schema validation
- SQL injection prevention
- XSS prevention
- CSRF protection

**CLI Input:**
- Argument validation
- Path traversal prevention
- Command injection prevention

### Secure Defaults

**Configuration:**
- API binds to localhost only
- Encryption optional but recommended
- No remote access by default
- No telemetry by default

**Permissions:**
- Plugins require explicit permissions
- Users grant permissions at install
- Permissions can be revoked

**Storage:**
- Database in user directory
- File system permissions restricted
- No world-readable files

## Security Best Practices

### Development

**Code Review:**
- All code reviewed before merge
- Security-focused review for sensitive code
- Two maintainer approval required

**Dependencies:**
- Regular dependency updates
- Security scanning (cargo-audit, npm audit)
- Minimal dependency usage

**Testing:**
- Security-focused tests
- Fuzz testing (future)
- Penetration testing (before v1.0)

### Deployment

**Code Signing:**
- Windows binaries signed
- macOS binaries signed and notarized
- Linux packages signed

**Distribution:**
- Secure build pipeline
- Reproducible builds (future)
- Hash verification

**Updates:**
- Signed updates
- Update verification
- Rollback capability

### Operations

**Monitoring:**
- Security event logging
- Anomaly detection (future)
- Alert on suspicious activity

**Incident Response:**
- Security incident policy
- Response team
- Communication plan

## Vulnerability Management

### Reporting Vulnerabilities

**How to Report:**
- Email: security@openpaste.org (when available)
- Private GitHub issue (when available)
- PGP key for encryption (when available)

**What to Include:**
- Vulnerability description
- Steps to reproduce
- Impact assessment
- Proof of concept (optional)

**Response Timeline:**
- Acknowledgment: Within 48 hours
- Initial assessment: Within 7 days
- Fix timeline: Based on severity

### Severity Levels

**Critical:**
- Remote code execution
- Data exposure without user interaction
- Bypass of encryption

**High:**
- Local code execution
- Data exposure with user interaction
- Privilege escalation

**Medium:**
- Information disclosure
- Denial of service
- Authentication bypass

**Low:**
- Minor information disclosure
- UI issues
- Configuration errors

### Disclosure Policy

**Coordinated Disclosure:**
- Work with reporter on timeline
- Fix before public disclosure
- Credit reporter in release notes

**Public Disclosure:**
- After fix is released
- Include CVE (if applicable)
- Include mitigation guidance

## Compliance

### Privacy

**Data Minimization:**
- Collect only necessary data
- Optional telemetry
- User control over data

**User Rights:**
- Right to access data
- Right to delete data
- Right to export data

**Transparency:**
- Privacy policy
- Data usage documentation
- Open source code

### Regulations

**GDPR:**
- Data protection by design
- User consent
- Data portability
- Right to be forgotten

**CCPA:**
- Privacy notice
- Opt-out mechanism
- Data deletion

## Security Audits

### Pre-Release Audits

**Internal Audit:**
- Code review
- Architecture review
- Threat modeling

**External Audit:**
- Third-party security firm
- Penetration testing
- Vulnerability assessment

### Ongoing Audits

**Dependency Scanning:**
- Automated scanning in CI
- Weekly manual review
- Immediate action on CVEs

**Code Scanning:**
- Static analysis
- Secret scanning
- Pattern matching

## Security Checklist

### Before Release

**Code:**
- [ ] All code reviewed
- [ ] Security tests pass
- [ ] No known vulnerabilities
- [ ] Dependencies updated

**Configuration:**
- [ ] Secure defaults
- [ ] No hardcoded secrets
- [ ] Proper file permissions
- [ ] Encryption enabled by default

**Documentation:**
- [ ] Security documentation updated
- [ ] Privacy policy updated
- [ ] Release notes include security info

### After Release

**Monitoring:**
- [ ] Security events monitored
- [ ] Anomaly detection enabled
- [ ] Incident response ready

**Updates:**
- [ ] Dependency updates monitored
- [ ] Security patches prioritized
- [ ] User communication plan

## Known Limitations

### Current Limitations

**Clipboard Access:**
- Other applications can read clipboard
- No clipboard isolation
- Platform limitation

**Memory:**
- Keys in memory while unlocked
- Vulnerable to memory dumps
- Mitigation: auto-lock

**Plugins:**
- WASM sandbox escape (theoretical)
- Side-channel attacks
- Mitigation: permission limits

### Future Mitigations

**Clipboard:**
- OS-level clipboard isolation (future)
- Secure clipboard (future)

**Memory:**
- Hardware security module (future)
- Secure enclave (future)

**Plugins:**
- Enhanced sandbox (future)
- Formal verification (future)

## Security Resources

### Documentation

**Internal:**
- [ENCRYPTION.md](ENCRYPTION.md) - Encryption design
- [IPC.md](IPC.md) - IPC security
- [PLUGIN_SDK.md](PLUGIN_SDK.md) - Plugin security

**External:**
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Rust Security Guidelines](https://doc.rust-lang.org/nomicon/security.html)

### Tools

**Scanning:**
- cargo-audit
- npm audit
- cargo-deny
- trivy

**Testing:**
- cargo-fuzz
- AFL
- libFuzzer

**Monitoring:**
- auditd (Linux)
- Event Tracing (Windows)
- Unified Logging (macOS)

## Security Team

### Responsibilities

**Security Lead:**
- Security architecture
- Vulnerability management
- Security audits

**Security Reviewers:**
- Code review
- Threat modeling
- Incident response

### Contact

**Security Issues:**
- Email: security@openpaste.org (when available)
- GitHub Security Advisory (when available)

**General Security:**
- GitHub Discussions
- Issues (non-sensitive)

## Incident Response

### Incident Types

**Data Breach:**
- Unauthorized data access
- Data exfiltration
- Data corruption

**Compromise:**
- Code injection
- Supply chain attack
- Credential theft

**Denial of Service:**
- Service unavailable
- Resource exhaustion
- Attack on availability

### Response Process

**Detection:**
- Monitoring alerts
- User reports
- Automated detection

**Containment:**
- Isolate affected systems
- Disable affected services
- Preserve evidence

**Eradication:**
- Remove threat
- Patch vulnerabilities
- Clean systems

**Recovery:**
- Restore from backup
- Verify integrity
- Resume operations

**Lessons Learned:**
- Post-mortem analysis
- Process improvement
- Documentation update

## Security FAQ

### Is my clipboard data encrypted?

Yes, if encryption is enabled. Data is encrypted at rest using AES-256-GCM. Encryption is optional but recommended.

### Can other applications read my clipboard?

Yes, this is a platform limitation. OpenPaste cannot prevent other applications from reading the clipboard. We recommend using the system clipboard manager or secure clipboard features when available.

### Are plugins safe?

Plugins run in a WASM sandbox with limited permissions. Users must grant permissions explicitly. However, no system is 100% secure, so only install plugins from trusted sources.

### Is the REST API secure?

By default, the API binds to localhost only, which provides network isolation. For remote access, TLS can be enabled. API tokens are used for authentication.

### What happens if I forget my master password?

If encryption is enabled and you forget your master password, your data cannot be recovered. This is intentional for security. We recommend using a password manager to store your master password.

### Does OpenPaste send data to the cloud?

No, OpenPaste does not send data to the cloud by default. All data is stored locally. Optional cloud sync can be enabled, but it requires explicit configuration.

### How do I report a security vulnerability?

Please report security vulnerabilities privately via email to security@openpaste.org (when available) or via GitHub Security Advisory. Do not report vulnerabilities in public issues.

## References

### Security Standards

- [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [ISO 27001](https://www.iso.org/standard/27001)

### Cryptography Standards

- [NIST Cryptographic Standards](https://csrc.nist.gov/projects/cryptographic-standards-and-guidelines)
- [RFC 9106 - Argon2](https://datatracker.ietf.org/doc/html/rfc9106)
- [NIST SP 800-38D - GCM](https://csrc.nist.gov/publications/detail/sp/800-38d/final)

### Platform Security

- [Windows Security](https://docs.microsoft.com/en-us/windows/security/)
- [Linux Security](https://www.linux.org/docs/)
- [macOS Security](https://support.apple.com/guide/security/welcome/mac)
