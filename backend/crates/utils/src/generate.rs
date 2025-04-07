use bip39::{Mnemonic, Language};
use tiny_hderive::bip32::ExtendedPrivKey;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use tiny_keccak::{Keccak, Hasher};
use hex;
// Required for PBKDF2 implementation
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;

#[derive(Debug, Clone)]
pub struct EthereumWallet {
    mnemonic: Mnemonic,
    private_key: [u8; 32],
    public_key: [u8; 65],
    address: String,
}

impl EthereumWallet {
    pub fn new() -> Self {
        let mnemonic = Self::generate_mnemonic();
        let private_key = Self::derive_private_key(&mnemonic);
        let public_key = Self::derive_public_key(&private_key);
        let address = Self::derive_address(&public_key);
        
        Self {
            mnemonic,
            private_key,
            public_key,
            address,
        }
    }

    // We're using the same method to generate mnemonic as your code originally did
    fn generate_mnemonic() -> Mnemonic {
        let mut rng = bip39::rand::thread_rng();
        Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap()
    }

    // Method to directly create a wallet from seed bytes
    // This bypasses the need to parse a mnemonic phrase
    pub fn from_seed(seed: &[u8]) -> Result<Self, &'static str> {
        if seed.len() < 32 {
            return Err("Seed too short");
        }
        
        let path = "m/44'/60'/0'/0/0";
        let ext = match ExtendedPrivKey::derive(seed, path) {
            Ok(key) => key,
            Err(_) => return Err("Failed to derive extended key from seed"),
        };
        
        let private_key = ext.secret();
        let public_key = Self::derive_public_key(&private_key);
        let address = Self::derive_address(&public_key);
        
        // We still need a mnemonic for the struct, but since we can't parse it,
        // we'll generate a new one (this won't match the original phrase)
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
        
        Ok(Self {
            mnemonic,  // Note: This won't match the seed that was used
            private_key,
            public_key,
            address,
        })
    }

    // Proper BIP-39 implementation of mnemonic to seed conversion using PBKDF2
    pub fn seed_from_phrase(phrase: &str, passphrase: &str) -> Vec<u8> {
        let salt = format!("mnemonic{}", passphrase);
        
        // Create a 64-byte (512-bit) output buffer
        let mut seed = vec![0u8; 64];
        
        // Perform PBKDF2 derivation
        let _ = pbkdf2::<Hmac<Sha512>>(
            phrase.as_bytes(),
            salt.as_bytes(),
            2048, // BIP-39 specifies 2048 rounds
            &mut seed
        );
        
        seed
    }

    fn derive_private_key(mnemonic: &Mnemonic) -> [u8; 32] {
        let path = "m/44'/60'/0'/0/0";
        let ext = ExtendedPrivKey::derive(mnemonic.to_seed("").as_ref(), path).unwrap();
        ext.secret()
    }

    fn derive_public_key(private_key: &[u8; 32]) -> [u8; 65] {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(private_key).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        public_key.serialize_uncompressed()
    }

    fn derive_address(public_key: &[u8; 65]) -> String {
        // Remove the first byte (0x04) which indicates uncompressed key
        let key_without_prefix = &public_key[1..];
        
        // Create keccak-256 hash
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(key_without_prefix);
        hasher.finalize(&mut hash);
        
        // Take last 20 bytes as Ethereum address
        let address = &hash[12..32];
        format!("0x{}", hex::encode(address))
    }

    // Getters
    pub fn mnemonic_phrase(&self) -> String {
        self.mnemonic.to_string()
    }

    pub fn private_key_hex(&self) -> String {
        hex::encode(self.private_key)
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn public_key(&self) -> String {
        hex::encode(self.public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wallet_creation() {
        let wallet = EthereumWallet::new();
        
        // Check that the mnemonic has 24 words
        assert_eq!(wallet.mnemonic_phrase().split_whitespace().count(), 24);
        
        // Check that private key is 32 bytes (64 hex chars)
        assert_eq!(wallet.private_key_hex().len(), 64);
        
        // Check that the address starts with "0x" and is 42 chars long
        assert!(wallet.address().starts_with("0x"));
        assert_eq!(wallet.address().len(), 42);
        
        // Check that public key is 65 bytes (130 hex chars)
        assert_eq!(wallet.public_key().len(), 130);
    }
    
    #[test]
    fn test_address_derivation() {
        // Use a known private key and expected address for deterministic testing
        let private_key = [
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
            0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
            0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09,
        ];
        
        let public_key = EthereumWallet::derive_public_key(&private_key);
        let address = EthereumWallet::derive_address(&public_key);
        
        // Address derivation should be deterministic
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
        
        // Generate the address again and ensure it's the same
        let address2 = EthereumWallet::derive_address(&public_key);
        assert_eq!(address, address2);
    }
    
    #[test]
    fn test_from_seed() {
        // Create a dummy 64-byte seed
        let seed = [0u8; 64];
        
        // Create wallet from seed
        let wallet = EthereumWallet::from_seed(&seed).expect("Failed to create wallet from seed");
        
        // Basic validation
        assert_eq!(wallet.mnemonic_phrase().split_whitespace().count(), 24);
        assert!(wallet.address().starts_with("0x"));
        assert_eq!(wallet.address().len(), 42);
    }
    
    #[test]
    fn test_seed_from_phrase() {
        // Test vector from BIP-39 specification
        // https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki#Test_vectors
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let passphrase = "";
        
        let seed = EthereumWallet::seed_from_phrase(phrase, passphrase);
        
        // Expected result from the BIP-39 test vector
        let expected_hex = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";
        let expected = hex::decode(expected_hex).unwrap();
        
        assert_eq!(seed, expected);
    }
    
    #[test]
    fn test_seed_from_phrase_with_passphrase() {
        // Test with a passphrase
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let passphrase = "TestingPassphrase";
        
        let seed = EthereumWallet::seed_from_phrase(phrase, passphrase);
        
        // Seed should be different with a passphrase
        let seed_no_passphrase = EthereumWallet::seed_from_phrase(phrase, "");
        assert_ne!(seed, seed_no_passphrase);
        
        // Seed should be 64 bytes
        assert_eq!(seed.len(), 64);
    }
    
    #[test]
    fn test_deterministic_wallet_generation() {
        // Create two wallets from the same mnemonic
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = EthereumWallet::seed_from_phrase(phrase, "");
        
        let wallet1 = EthereumWallet::from_seed(&seed).expect("Failed to create wallet1");
        let wallet2 = EthereumWallet::from_seed(&seed).expect("Failed to create wallet2");
        
        // Both wallets should have the same address and private key
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.private_key_hex(), wallet2.private_key_hex());
    }
    
    #[test]
    fn test_short_seed_error() {
        // Test with a seed that's too short
        let short_seed = [0u8; 16];
        let result = EthereumWallet::from_seed(&short_seed);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Seed too short");
    }
}