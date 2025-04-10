# Security Guide for Wallet Data

## Key Management Architecture

This wallet service uses a multi-tier key management system powered by HashiCorp Vault:

### 1. User Layer: PIN Protection
- Each wallet is protected by a 6-digit PIN known only to the user
- PIN is used to derive an encryption key via PBKDF2 with a unique salt
- PIN is never stored, only the user knows it

### 2. Application Layer: Data Encryption Keys (DEK)
- Each wallet gets a unique Data Encryption Key (DEK)
- The PIN-encrypted data is further encrypted with this DEK
- DEKs are encrypted by the master key

### 3. System Layer: Master Key in Vault
- The master key is stored securely in HashiCorp Vault, not in application memory
- Vault provides access control, auditing, and secure storage
- Access to the master key requires authentication and authorization

## Security Benefits

1. **Defense in Depth**: Multiple encryption layers protect against various threat vectors
2. **Secure Key Storage**: Master keys are stored in Vault, a dedicated secrets management system
3. **Audit Trail**: All key access is logged for security analysis
4. **Access Control**: Fine-grained policies control which services can access encryption keys
5. **Key Rotation**: Master keys can be rotated without re-encrypting all wallet data
6. **Compliance**: Meets industry standards for cryptographic key management

## Secure Implementation Checklist
Before deploying to production, ensure:

- [x] Private keys are encrypted with multiple layers (PIN, DEK, Master Key)
- [x] A secure key management solution is in place (HashiCorp Vault)
- [ ] Database connections use TLS with strong authentication
- [ ] Proper access controls are implemented
- [x] All sensitive operations are audited
- [ ] Rate limiting is applied to prevent abuse
- [ ] Regular security reviews are scheduled

## Vault Configuration

The system uses the following Vault configuration:

- **Secrets Engine**: KV Version 2
- **Authentication**: UserPass method for service authentication
- **Policies**: Limited access to only required paths
- **Data Structure**:
  - Master keys: `kv/crypto/master_keys/{key_id}`
  - Wallet encryption data stored in database

## Emergency Procedures

In case of a security incident:

1. Revoke access to the compromised Vault tokens
2. Rotate the affected keys
3. Notify the security team immediately
4. Preserve evidence for investigation
5. Follow the incident response plan

## Testing Security Measures

Use these approaches to validate security measures:

1. Conduct penetration testing of the wallet service
2. Perform regular code security reviews focusing on the cryptographic implementation
3. Verify key access procedures with Vault audit logs
4. Use automated scanning tools for vulnerability detection
5. Verify encryption implementations with security experts

## Production Hardening

For production deployment:

1. Enable TLS for Vault communication
2. Configure a robust storage backend (Consul or cloud provider)
3. Implement auto-unseal with cloud KMS
4. Set up high availability for Vault
5. Use more robust authentication (such as TLS certificates)
6. Implement proper network security controls
7. Configure regular key rotation