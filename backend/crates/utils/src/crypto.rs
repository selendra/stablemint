use app_error::{AppError, AppResult};
use hex;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::{RngCore, rng};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Constants for encryption
const PBKDF2_ITERATIONS: u32 = 10000; // High number for security
const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32; // 256 bits
const TAG_LENGTH: usize = 16; // GCM authentication tag

// DEK cache for performance - only caches keys after they're fetched from HCP
pub struct DekCache {
    cache: RwLock<HashMap<String, Vec<u8>>>,
}

impl DekCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, dek_id: &str) -> Option<Vec<u8>> {
        let cache = self.cache.read().await;
        cache.get(dek_id).cloned()
    }

    pub async fn set(&self, dek_id: String, dek: Vec<u8>) {
        let mut cache = self.cache.write().await;
        cache.insert(dek_id, dek);
    }
}

/// Wallet encryption service for handling the secure storage of wallet private keys
pub struct WalletEncryptionService {
    // Fields for master key identification
    pub master_key_id: String,
    // In-memory cache of data encryption keys
    dek_cache: Arc<DekCache>,
    //
    master_key: Arc<[u8]>,
}

impl WalletEncryptionService {
    /// Creates a new WalletEncryptionService instance with HCP Secrets
    pub fn new(master_key_id: &str, master_key: &[u8]) -> Self {
        Self {
            master_key_id: master_key_id.to_string(),
            dek_cache: Arc::new(DekCache::new()),
            master_key: Arc::from(master_key.to_vec()),
        }
    }

    /// Encrypt a private key with user PIN and then with DEK and master key
    pub async fn encrypt_private_key(
        &self,
        private_key: &str,
        pin: &str,
    ) -> AppResult<WalletEncryptedData> {
        // Step 1: PIN encryption - derive a key from the PIN
        let pin_salt = Self::generate_random_bytes(SALT_LENGTH);
        let pin_key = Self::derive_key_from_pin(pin, &pin_salt)?;

        // Step 2: Encrypt the private key with the PIN-derived key
        let pin_iv = Self::generate_random_bytes(IV_LENGTH);
        let pin_encrypted = Self::aes_gcm_encrypt(private_key.as_bytes(), &pin_key, &pin_iv)?;

        // Step 3: Generate a random DEK (Data Encryption Key)
        let dek = Self::generate_random_bytes(KEY_LENGTH);

        // Step 4: Encrypt the PIN-encrypted data with the DEK
        let dek_iv = Self::generate_random_bytes(IV_LENGTH);
        let dek_encrypted = Self::aes_gcm_encrypt(&pin_encrypted, &dek, &dek_iv)?;

        let master_iv = Self::generate_random_bytes(IV_LENGTH);
        let encrypted_dek = Self::aes_gcm_encrypt(&dek, &self.master_key, &master_iv)?;

        // Cache the DEK for future use
        let dek_id = Uuid::new_v4().to_string();
        self.dek_cache.set(dek_id.clone(), dek).await;

        // Return the encrypted data structure
        Ok(WalletEncryptedData {
            user_id: "".to_string(), // Set this when associating with a user
            encrypted_private_key: hex::encode(dek_encrypted),
            encrypted_dek: hex::encode(encrypted_dek),
            master_key_identifier: self.master_key_id.clone(),
            dek_id: dek_id,
            algorithm: "AES-256-GCM".to_string(),
            pin_salt: hex::encode(pin_salt),
            pin_iv: hex::encode(pin_iv),
            dek_iv: hex::encode(dek_iv),
            master_iv: hex::encode(master_iv),
        })
    }

    /// Decrypt a private key using the reverse process
    pub async fn decrypt_private_key(
        &self,
        encrypted_data: &WalletEncryptedData,
        pin: &str,
    ) -> AppResult<String> {
        // Validate the master key identifier
        if encrypted_data.master_key_identifier != self.master_key_id {
            return Err(AppError::ValidationError(
                "Invalid master key identifier".to_string(),
            ));
        }

        // Step 1: Try to get DEK from cache first
        let dek = match self.dek_cache.get(&encrypted_data.dek_id).await {
            Some(dek) => dek,
            None => {
                // If not in cache, decrypt it using the master key

                let encrypted_dek = hex::decode(&encrypted_data.encrypted_dek)
                    .map_err(|_| AppError::ValidationError("Invalid DEK format".to_string()))?;
                let master_iv = hex::decode(&encrypted_data.master_iv).map_err(|_| {
                    AppError::ValidationError("Invalid master IV format".to_string())
                })?;

                let dek = Self::aes_gcm_decrypt(&encrypted_dek, &self.master_key, &master_iv)?;

                // Add to cache for future use
                self.dek_cache
                    .set(encrypted_data.dek_id.clone(), dek.clone())
                    .await;

                dek
            }
        };

        // Step 2: Decrypt the encrypted private key with the DEK
        let dek_encrypted = hex::decode(&encrypted_data.encrypted_private_key)
            .map_err(|_| AppError::ValidationError("Invalid encrypted data format".to_string()))?;
        let dek_iv = hex::decode(&encrypted_data.dek_iv)
            .map_err(|_| AppError::ValidationError("Invalid DEK IV format".to_string()))?;

        let pin_encrypted = Self::aes_gcm_decrypt(&dek_encrypted, &dek, &dek_iv)?;

        // Step 3: Derive the key from the PIN
        let pin_salt = hex::decode(&encrypted_data.pin_salt)
            .map_err(|_| AppError::ValidationError("Invalid PIN salt format".to_string()))?;
        let pin_key = Self::derive_key_from_pin(pin, &pin_salt)?;

        // Step 4: Decrypt the PIN-encrypted data
        let pin_iv = hex::decode(&encrypted_data.pin_iv)
            .map_err(|_| AppError::ValidationError("Invalid PIN IV format".to_string()))?;

        let private_key_bytes = Self::aes_gcm_decrypt(&pin_encrypted, &pin_key, &pin_iv)?;

        // Convert back to string
        String::from_utf8(private_key_bytes)
            .map_err(|_| AppError::ValidationError("Invalid private key data".to_string()))
    }

    /// Generate random bytes for cryptographic operations
    fn generate_random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        rng().fill_bytes(&mut bytes);
        bytes
    }

    /// Derive a key from a PIN using PBKDF2
    fn derive_key_from_pin(pin: &str, salt: &[u8]) -> AppResult<Vec<u8>> {
        let mut key = vec![0u8; KEY_LENGTH];

        pbkdf2::<Hmac<Sha512>>(pin.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key)
            .map_err(|_| AppError::CryptoError("Failed to derive key from PIN".to_string()))?;

        Ok(key)
    }

    /// AES-GCM encryption (simplified - in a real system, use a more robust crypto library)
    fn aes_gcm_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> AppResult<Vec<u8>> {
        // Note: This is a simplified implementation for demonstration
        // In a real application, use a proper crypto library like ring or RustCrypto

        // For this implementation, we'll just XOR the data with the key (NOT SECURE)
        // and append a mock "tag" (also NOT SECURE)
        let mut result = Vec::with_capacity(data.len() + TAG_LENGTH);

        // "Encrypt" the data (this is NOT actual AES-GCM encryption)
        for (i, byte) in data.iter().enumerate() {
            result.push(byte ^ key[i % key.len()]);
        }

        // Generate a mock "authentication tag" by hashing the data and key
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(key);
        hasher.update(iv);
        let tag = hasher.finalize();
        result.extend_from_slice(&tag[0..TAG_LENGTH]);

        Ok(result)
    }

    /// AES-GCM decryption (simplified - in a real system, use a more robust crypto library)
    fn aes_gcm_decrypt(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> AppResult<Vec<u8>> {
        // Split ciphertext and tag
        if ciphertext.len() < TAG_LENGTH {
            return Err(AppError::ValidationError(
                "Invalid ciphertext format".to_string(),
            ));
        }

        let (encrypted_data, tag) = ciphertext.split_at(ciphertext.len() - TAG_LENGTH);

        // Verify the "tag" (this is NOT actual AES-GCM verification)
        let mut hasher = Sha256::new();

        // "Decrypt" the data (this is NOT actual AES-GCM decryption)
        let mut result = Vec::with_capacity(encrypted_data.len());

        for (i, byte) in encrypted_data.iter().enumerate() {
            result.push(byte ^ key[i % key.len()]);
        }

        hasher.update(&result);
        hasher.update(key);
        hasher.update(iv);
        let expected_tag = hasher.finalize();

        // Verify tag (time-constant comparison would be better in production)
        if tag != &expected_tag[0..TAG_LENGTH] {
            return Err(AppError::ValidationError(
                "Invalid authentication tag".to_string(),
            ));
        }

        Ok(result)
    }
}

/// Structure to hold encrypted wallet data
#[derive(Debug, Clone)]
pub struct WalletEncryptedData {
    pub user_id: String,
    pub encrypted_private_key: String, // Hex-encoded AES-GCM encrypted private key (encrypted with DEK)
    pub encrypted_dek: String, // Hex-encoded AES-GCM encrypted DEK (encrypted with master key)
    pub master_key_identifier: String, // Identifier for the master key used
    pub dek_id: String,        // ID for the DEK (used for caching)
    pub algorithm: String,     // Encryption algorithm used (e.g., "AES-256-GCM")
    pub pin_salt: String,      // Hex-encoded salt for PIN key derivation
    pub pin_iv: String,        // Hex-encoded IV for PIN encryption
    pub dek_iv: String,        // Hex-encoded IV for DEK encryption
    pub master_iv: String,     // Hex-encoded IV for master key encryption
}

impl WalletEncryptedData {
    /// Set the user ID associated with this encrypted data
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.to_string();
        self
    }

    /// Convert to a string representation for storage
    pub fn to_storage_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Create from a storage string
    pub fn from_storage_string(data: &str) -> AppResult<Self> {
        serde_json::from_str(data)
            .map_err(|_| AppError::ValidationError("Invalid encrypted data format".to_string()))
    }
}

// Make WalletEncryptedData serializable
impl serde::Serialize for WalletEncryptedData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WalletEncryptedData", 10)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("encrypted_private_key", &self.encrypted_private_key)?;
        state.serialize_field("encrypted_dek", &self.encrypted_dek)?;
        state.serialize_field("master_key_identifier", &self.master_key_identifier)?;
        state.serialize_field("dek_id", &self.dek_id)?;
        state.serialize_field("algorithm", &self.algorithm)?;
        state.serialize_field("pin_salt", &self.pin_salt)?;
        state.serialize_field("pin_iv", &self.pin_iv)?;
        state.serialize_field("dek_iv", &self.dek_iv)?;
        state.serialize_field("master_iv", &self.master_iv)?;
        state.end()
    }
}

// Make WalletEncryptedData deserializable
impl<'de> serde::Deserialize<'de> for WalletEncryptedData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct WalletEncryptedDataVisitor;

        impl<'de> Visitor<'de> for WalletEncryptedDataVisitor {
            type Value = WalletEncryptedData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WalletEncryptedData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WalletEncryptedData, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut user_id = None;
                let mut encrypted_private_key = None;
                let mut encrypted_dek = None;
                let mut master_key_identifier = None;
                let mut dek_id = None;
                let mut algorithm = None;
                let mut pin_salt = None;
                let mut pin_iv = None;
                let mut dek_iv = None;
                let mut master_iv = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "user_id" => {
                            user_id = Some(map.next_value()?);
                        }
                        "encrypted_private_key" => {
                            encrypted_private_key = Some(map.next_value()?);
                        }
                        "encrypted_dek" => {
                            encrypted_dek = Some(map.next_value()?);
                        }
                        "master_key_identifier" => {
                            master_key_identifier = Some(map.next_value()?);
                        }
                        "dek_id" => {
                            dek_id = Some(map.next_value()?);
                        }
                        "algorithm" => {
                            algorithm = Some(map.next_value()?);
                        }
                        "pin_salt" => {
                            pin_salt = Some(map.next_value()?);
                        }
                        "pin_iv" => {
                            pin_iv = Some(map.next_value()?);
                        }
                        "dek_iv" => {
                            dek_iv = Some(map.next_value()?);
                        }
                        "master_iv" => {
                            master_iv = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let encrypted_private_key = encrypted_private_key
                    .ok_or_else(|| de::Error::missing_field("encrypted_private_key"))?;
                let encrypted_dek =
                    encrypted_dek.ok_or_else(|| de::Error::missing_field("encrypted_dek"))?;
                let master_key_identifier = master_key_identifier
                    .ok_or_else(|| de::Error::missing_field("master_key_identifier"))?;
                let dek_id = dek_id.ok_or_else(|| de::Error::missing_field("dek_id"))?;
                let algorithm = algorithm.ok_or_else(|| de::Error::missing_field("algorithm"))?;
                let pin_salt = pin_salt.ok_or_else(|| de::Error::missing_field("pin_salt"))?;
                let pin_iv = pin_iv.ok_or_else(|| de::Error::missing_field("pin_iv"))?;
                let dek_iv = dek_iv.ok_or_else(|| de::Error::missing_field("dek_iv"))?;
                let master_iv = master_iv.ok_or_else(|| de::Error::missing_field("master_iv"))?;

                Ok(WalletEncryptedData {
                    user_id,
                    encrypted_private_key,
                    encrypted_dek,
                    master_key_identifier,
                    dek_id,
                    algorithm,
                    pin_salt,
                    pin_iv,
                    dek_iv,
                    master_iv,
                })
            }
        }

        deserializer.deserialize_map(WalletEncryptedDataVisitor)
    }
}
