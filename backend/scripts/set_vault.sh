#!/bin/bash
# Script to initialize and configure HashiCorp Vault for use with the wallet service

# Exit on any error
set -e

echo "Setting up HashiCorp Vault for wallet encryption key management..."

# Create required directories
mkdir -p vault/config vault/data vault/logs

# Check if vault.json already exists, if not, create it
if [ ! -f vault/config/vault.json ]; then
    echo "Creating Vault configuration file..."
    cat > vault/config/vault.json << EOL
{
  "storage": {
    "file": {
      "path": "/vault/data"
    }
  },
  "listener": {
    "tcp": {
      "address": "0.0.0.0:8200",
      "tls_disable": 1
    }
  },
  "ui": true,
  "disable_mlock": true
}
EOL
fi

# Start Vault if it's not already running
if ! docker ps | grep -q vault; then
    echo "Starting Vault container..."
    docker-compose up -d vault
    # Wait for Vault to start
    sleep 5
fi

# Check Vault status
VAULT_STATUS=$(docker exec vault vault status -format=json 2>/dev/null || echo '{"initialized": false}')
INITIALIZED=$(echo $VAULT_STATUS | grep -o '"initialized":[^,}]*' | grep -o '[^:]*$' | tr -d ' "')

if [ "$INITIALIZED" == "false" ]; then
    echo "Initializing Vault..."
    INIT_OUTPUT=$(docker exec vault vault operator init -key-shares=3 -key-threshold=2 -format=json)
    
    # Save the unseal keys and root token to a secure file
    echo "$INIT_OUTPUT" > vault/init-keys.json
    chmod 600 vault/init-keys.json
    
    echo "Vault initialized. Unseal keys and root token saved to vault/init-keys.json"
    echo "IMPORTANT: Keep this file secure! In production, these should be stored securely and separately."
    
    # Extract unseal keys and root token
    UNSEAL_KEY_1=$(echo "$INIT_OUTPUT" | grep -o '"unseal_keys_b64":\[[^]]*\]' | grep -o '"[^"]*"' | sed -n 1p | tr -d '"')
    UNSEAL_KEY_2=$(echo "$INIT_OUTPUT" | grep -o '"unseal_keys_b64":\[[^]]*\]' | grep -o '"[^"]*"' | sed -n 2p | tr -d '"')
    ROOT_TOKEN=$(echo "$INIT_OUTPUT" | grep -o '"root_token":"[^"]*"' | grep -o '[^"]*$' | tr -d '"')
    
    # Unseal Vault
    echo "Unsealing Vault..."
    docker exec vault vault operator unseal "$UNSEAL_KEY_1"
    docker exec vault vault operator unseal "$UNSEAL_KEY_2"
else
    echo "Vault is already initialized."
    
    # Check if Vault is sealed
    SEALED=$(echo $VAULT_STATUS | grep -o '"sealed":[^,}]*' | grep -o '[^:]*$' | tr -d ' "')
    
    if [ "$SEALED" == "true" ]; then
        echo "Vault is sealed. Please unseal it manually using the unseal keys from vault/init-keys.json."
        echo "Example: docker exec vault vault operator unseal YOUR_UNSEAL_KEY"
        exit 1
    fi
    
    # Try to get the root token from the saved file
    if [ -f vault/init-keys.json ]; then
        ROOT_TOKEN=$(cat vault/init-keys.json | grep -o '"root_token":"[^"]*"' | grep -o '[^"]*$' | tr -d '"')
    else
        echo "Cannot find root token. Please provide it manually."
        read -p "Enter root token: " ROOT_TOKEN
    fi
fi

# Login to Vault
echo "Logging in to Vault..."
docker exec vault vault login "$ROOT_TOKEN"

# Enable KV secrets engine version 2 if not already enabled
echo "Enabling KV secrets engine..."
docker exec vault vault secrets enable -path=kv -version=2 kv 2>/dev/null || echo "KV secrets engine already enabled"

# Enable userpass auth method if not already enabled
echo "Enabling userpass authentication..."
docker exec vault vault auth enable userpass 2>/dev/null || echo "Userpass auth already enabled"

# Create policy for the wallet service
echo "Creating wallet-service policy..."
cat > vault/wallet-policy.hcl << EOL
# Policy for wallet service to manage encryption keys
path "kv/data/crypto/master_keys/*" {
  capabilities = ["create", "read", "update"]
}

path "kv/data/crypto/dek/*" {
  capabilities = ["create", "read", "update"]
}
EOL

docker exec -i vault vault policy write wallet-policy - < vault/wallet-policy.hcl

# Create a wallet-service user with the wallet-policy
echo "Creating wallet-service user..."
docker exec vault vault write auth/userpass/users/wallet-service \
    password=vault-password \
    policies=wallet-policy

echo "Creating test-user for development..."
docker exec vault vault write auth/userpass/users/test-user \
    password=test-password \
    policies=wallet-policy

echo "Vault setup complete! The wallet service can now use Vault for secure key management."
echo "Summary:"
echo "  - Vault UI: http://localhost:8200"
echo "  - Authentication: userpass method enabled"
echo "  - Service username: wallet-service"
echo "  - Service password: vault-password"
echo "  - KV secrets engine enabled at path: kv/"
echo "  - Wallet policy created for secure access to encryption keys"
echo ""
echo "IMPORTANT: For production use, configure proper TLS, use a more robust storage backend,"
echo "and implement a proper unsealing strategy such as auto-unseal with a cloud KMS."