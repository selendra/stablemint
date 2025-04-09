## Multi-Layer Wallet Encryption Implementation
I've implemented a comprehensive wallet security system that protects private keys using multiple encryption layers as you requested. Here's how it works:

### Key Security Concepts
The implementation uses a "tiered encryption" approach (also known as envelope encryption):

#### 1. User Layer - PIN Protection

User provides a 6-digit PIN to access their wallet
PIN is used to derive an encryption key via PBKDF2 with a unique salt
This ensures only someone with the PIN can access the private key


#### 2. Application Layer - Data Encryption Key (DEK)

Each wallet gets a unique Data Encryption Key
The PIN-encrypted data is further encrypted with this DEK
DEKs are tracked and cached for performance


#### 3. System Layer - Master Key

A master key encrypts all the DEKs
The master key is managed separately (ideally in a KMS or HSM)
This provides an additional security layer



## Security Benefits

- Defense in Depth: Compromising any single layer isn't enough to get the private key
- PIN Security: The PIN never leaves the user's control, and is never stored
- Key Isolation: The master key never directly encrypts sensitive data
- Key Rotation: Master keys can be rotated without re-encrypting all private keys
- Performance: DEK caching improves performance for frequent operations

## Implementation Details

#### 1. The WalletEncryptionService handles:

- Generating encryption keys
- Encrypting wallet data with multiple layers
- Decrypting wallet data when authorized by PIN
- Managing cryptographic material securely


####  2. The WalletService provides:

- Creating wallets with PIN protection
- Transferring funds that requires PIN verification
- Changing PINs while maintaining the same underlying keys
- PIN verification without exposing the private key


####  3. The GraphQL schema exposes user-friendly operations:

- Creating wallets with secure PINs
- Making transfers with PIN authorization
- Changing/verifying PINs