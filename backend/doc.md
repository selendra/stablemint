# GraphQL API Documentation

This document provides a comprehensive guide to interacting with the User and Wallet microservices via their GraphQL APIs.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [User Service](#user-service)
  - [Queries](#user-queries)
  - [Mutations](#user-mutations)
- [Wallet Service](#wallet-service)
  - [Queries](#wallet-queries)
  - [Mutations](#wallet-mutations)
- [Error Handling](#error-handling)
- [Security Considerations](#security-considerations)
- [Examples](#examples)

## Overview

The backend is built using a microservices architecture with two primary services:

1. **User Service**: Handles user registration, authentication, and profile management
2. **Wallet Service**: Manages cryptocurrency wallets, PIN protection, and fund transfers

Each service exposes a GraphQL API endpoint and requires JWT authentication for protected operations.

## Authentication

Most operations require authentication via a JWT token. The token is obtained during registration or login and should be included in the Authorization header of subsequent requests.

```
{
"Authorization":"Bearer <your_token>"
}
```

## User Service

Endpoint: `http://localhost:5000/graphql`

### User Queries

#### `me` - Get Current User Profile

Returns the profile of the authenticated user.

**Requires Authentication**: Yes

**Response Type**: `UserProfile`

**Example**:
```graphql
query {
  me {
    id
    name
    username
    email
    createdAt
    walletId
  }
}
```

### User Mutations

#### `register` - Create New User Account

**Input**: `RegisterInput`

**Response Type**: `AuthResponse`

**Example**:
```graphql
mutation {
  register(input: {
    name: "John Doe",
    username: "johndoe",
    email: "john@example.com",
    password: "Secure@123Password"
  }) {
    token
    user {
      id
      name
      username
      email
      createdAt
    }
  }
}
```

#### `login` - Authenticate User

**Input**: `LoginInput`

**Response Type**: `AuthResponse`

**Example**:
```graphql
mutation {
  login(input: {
    username: "johndoe",
    password: "Secure@123Password"
  }) {
    token
    user {
      id
      name
      username
      email
      createdAt
    }
  }
}
```

## Wallet Service

Endpoint: `http://localhost:5001/graphql`

### Wallet Queries

#### `myWallet` - Get Current User's Wallet

Returns the wallet information for the authenticated user.

**Requires Authentication**: Yes

**Response Type**: `WalletInfo`

**Example**:
```graphql
query {
  myWallet {
    id
    userEmail
    address
    createdAt
  }
}
```

#### `walletBalance` - Get Wallet Balance

**Parameters**:
- `walletId`: String (ID of the wallet)

**Requires Authentication**: Yes

**Response Type**: `Float`

**Example**:
```graphql
query {
  walletBalance(walletId: "wallets:c8e7f3ba-4b8d-4e41-a2eb-41df55d2") 
}
```

### Wallet Mutations

#### `createWallet` - Create a New Wallet

Creates a new cryptocurrency wallet for the current user, secured with a PIN.

**Input**: `CreateWalletInput`

**Requires Authentication**: Yes

**Response Type**: `WalletInfo`

**Example**:
```graphql
mutation {
  createWallet(input: {
    pin: "123456"
  }) {
    id
    userEmail
    address
    createdAt
  }
}
```

#### `transfer` - Transfer Funds

Transfer cryptocurrency from the user's wallet to another address. Requires PIN verification.

**Input**: `TransferInput`

**Requires Authentication**: Yes

**Response Type**: `String` (transaction hash)

**Example**:
```graphql
mutation {
  transfer(input: {
    toAddress: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
    amount: 0.5,
    pin: "123456"
  })
}
```

#### `changeWalletPin` - Change Wallet PIN

Updates the security PIN for the user's wallet.

**Input**: `ChangePinInput`

**Requires Authentication**: Yes

**Response Type**: `Boolean`

**Example**:
```graphql
mutation {
  changeWalletPin(input: {
    oldPin: "123456",
    newPin: "654321"
  })
}
```

#### `verifyWalletPin` - Verify Wallet PIN

Checks if the provided PIN is correct for the user's wallet.

**Parameters**:
- `pin`: String (6-digit PIN)

**Requires Authentication**: Yes

**Response Type**: `Boolean`

**Example**:
```graphql
mutation {
  verifyWalletPin(pin: "123456")
}
```

## Error Handling

The GraphQL API returns structured errors with the following properties:

- `message`: Human-readable error message
- `extensions`:
  - `code`: Error code identifier
  - `details`: Technical details (when available)
  - `help`: Additional guidance on fixing the error

Common error codes include:

- `VALIDATION_ERROR`: Invalid input data
- `AUTH_ERROR`: Authentication failure
- `FORBIDDEN`: Permission denied
- `NOT_FOUND`: Requested resource doesn't exist
- `RATE_LIMIT`: Too many requests
- `SERVER_ERROR`: Internal server issue

**Example error response**:
```json
{
  "errors": [
    {
      "message": "Login failed: The username or password you entered is incorrect",
      "extensions": {
        "code": "AUTH_ERROR",
        "details": "Authentication error: Login failed: The username or password you entered is incorrect",
        "help": "Please check your credentials and try again."
      }
    }
  ]
}
```

## Security Considerations

### Password Requirements

Passwords must:
- Be at least 8 characters long
- Contain at least one uppercase letter
- Contain at least one lowercase letter
- Contain at least one number
- Contain at least one special character (@$!%*?&)

### PIN Requirements

Wallet PINs must:
- Be exactly 6 digits
- Contain only numbers (0-9)

### Rate Limiting

The API implements rate limiting to prevent abuse:

- **General API requests**: 100 requests per minute
- **Login attempts**: 5 attempts per 5 minutes, with 15-minute lockout after failure

### Security Headers

All API responses include security headers such as:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`

## Examples

### Complete User Registration and Wallet Creation Flow

1. Register a new user:

```graphql
mutation {
  register(input: {
    name: "Alice Johnson",
    username: "alice",
    email: "alice@example.com",
    password: "Secure@123Password"
  }) {
    token
    user {
      id
      name
      username
      email
    }
  }
}
```

2. Use the token from registration to create a wallet:

```graphql
mutation {
  createWallet(input: {
    pin: "123456"
  }) {
    id
    address
    createdAt
  }
}
```

3. Check the wallet balance:

```graphql
query {
  walletBalance(walletId: "wallets:c8e7f3ba-4b8d-4e41-a2eb-41df55d2") 
}
```

4. Transfer funds using PIN for authentication:

```graphql
mutation {
  transfer(input: {
    toAddress: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
    amount: 0.5,
    pin: "123456"
  })
}
```

### Using the API with cURL

1. Register a user:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"query": "mutation { register(input: { name: \"Alice Johnson\", username: \"alice\", email: \"alice@example.com\", password: \"Secure@123Password\" }) { token user { id name username email } } }"}' \
  http://localhost:5000/graphql
```

2. Login with credentials:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"query": "mutation { login(input: { username: \"alice\", password: \"Secure@123Password\" }) { token user { id name username email } } }"}' \
  http://localhost:5000/graphql
```

3. Create a wallet (with authentication):

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"query": "mutation { createWallet(input: { pin: \"123456\" }) { id address createdAt } }"}' \
  http://localhost:5001/graphql
```

### Error Handling Examples

1. Invalid PIN format:

```graphql
mutation {
  createWallet(input: {
    pin: "12345"  # Too short, must be 6 digits
  }) {
    id
    address
  }
}
```

Response:
```json
{
  "errors": [
    {
      "message": "Validation error: PIN must be a 6-digit number",
      "extensions": {
        "code": "VALIDATION_ERROR",
        "details": "Validation error: PIN must be a 6-digit number",
        "help": "Please review your input and try again."
      }
    }
  ]
}
```

2. Authentication required:

```graphql
query {
  myWallet {
    id
    address
  }
}
```

Response (without token):
```json
{
  "errors": [
    {
      "message": "Authentication required. Please log in to view your wallet.",
      "extensions": {
        "code": "AUTH_ERROR",
        "details": "Authentication required. Please log in to view your wallet.",
        "help": "Please log in to access this resource."
      }
    }
  ]
}
```
