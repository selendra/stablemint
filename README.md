# StableMint: Tokenized Asset Platform

A comprehensive platform for creating, minting, and swapping tokenized assets backed by a stablecoin. This platform provides a full ecosystem for tokenized asset management with whitelisting capabilities, factory pattern for token creation, and a swap mechanism.

## Table of Contents

- [Overview](#overview)
- [Smart Contracts](#smart-contracts)
  - [StableCoin](#stablecoin)
  - [ERC20Token](#erc20token)
  - [ERC20Factory](#erc20factory)
  - [TokenSwap](#tokenswap)
- [Features](#features)
- [Getting Started](#getting-started)
- [Roles and Permissions](#roles-and-permissions)
- [Usage Examples](#usage-examples)
- [Security Considerations](#security-considerations)
- [License](#license)

## Overview

This platform enables the creation of tokenized assets that are backed by a stablecoin. It implements a factory pattern for token creation, with each token having a defined ratio to the stablecoin. The system includes sophisticated access control with various roles, whitelist functionality for compliance, and a swap mechanism for exchanging tokens with the backing stablecoin.

## Smart Contracts

### StableCoin

The StableCoin contract is an ERC20 token with added functionality:

- **Whitelisting**: Only whitelisted addresses can send or (optionally) receive the stablecoin
- **Role-Based Access Control**: Different permissions for minting, burning, and managing whitelist
- **Pausable**: Can be paused/unpaused by authorized addresses

Key roles:
- `DEFAULT_ADMIN_ROLE`: Can manage other roles
- `ADMIN_ROLE`: Can manage whitelist policy
- `PAUSER_ROLE`: Can pause/unpause transfers
- `MINTER_ROLE`: Can mint new tokens
- `BURNER_ROLE`: Can burn tokens
- `WHITELIST_MANAGER_ROLE`: Can manage the whitelist

### ERC20Token

A standard ERC20 token created by the factory with:

- **Controlled minting/burning**: Only the factory can mint/burn these tokens
- **Role-Based Access Control**: Admin and pauser roles for management
- **Pausable**: Can be paused/unpaused by authorized addresses

### ERC20Factory

The factory contract that creates and manages ERC20 tokens:

- **Token Creation**: Creates new ERC20 tokens with specified parameters
- **Token Ratio Management**: Defines how many tokens are minted per stablecoin
- **Minting Capability**: Mints tokens based on the stablecoin backing

Key roles:
- `FACTORY_ADMIN_ROLE`: Can manage factory settings
- `TOKEN_CREATOR_ROLE`: Can create new tokens
- `FACTORY_MINTER_ROLE`: Can mint tokens
- `RATIO_MANAGER_ROLE`: Can manage token ratios

### TokenSwap

A swap contract that allows exchange between the stablecoin and created tokens:

- **Bidirectional Swaps**: Convert stablecoin to tokens and vice versa
- **Fee Management**: Configurable fee percentage with a designated collector
- **Whitelist Enforcement**: Only whitelisted users can perform swaps
- **Emergency Withdrawal**: Admin can withdraw tokens in emergency

Key roles:
- `ADMIN_ROLE`: Can manage swap settings and perform emergency withdrawals
- `PAUSER_ROLE`: Can pause/unpause the swap functionality
- `FEE_MANAGER_ROLE`: Can update fee percentages

## Features

- **Compliance Ready**: Whitelist functionality ensures only approved accounts can transact
- **Flexible Token Creation**: Create multiple asset tokens with different ratios
- **Secure Architecture**: Built with OpenZeppelin contracts and reentrancy protection
- **Fee System**: Configurable fees for swap operations
- **Emergency Controls**: Pause functionality and emergency withdrawals
- **Role-Based Security**: Fine-grained permissions across all contracts

## Getting Started

### Prerequisites

- Solidity ^0.8.20
- OpenZeppelin Contracts
- An Ethereum development environment (Truffle, Hardhat, etc.)

### Deployment Sequence

1. Deploy the `StableCoin` contract with desired name, symbol, and initial supply
2. Deploy the `ERC20Factory` with the stablecoin address
3. Deploy the `TokenSwap` contract with addresses for stablecoin, factory, admin, fee collector, and initial fee

### Initial Setup

1. Grant necessary roles to administrative addresses
2. Add required addresses to the stablecoin whitelist
3. Create initial tokens through the factory with appropriate ratios

## Roles and Permissions

The system uses OpenZeppelin's AccessControl with the following roles:

### StableCoin Roles
- `DEFAULT_ADMIN_ROLE`: Manage roles
- `ADMIN_ROLE`: General admin operations
- `PAUSER_ROLE`: Pause/unpause transfers
- `MINTER_ROLE`: Mint new tokens
- `BURNER_ROLE`: Burn tokens
- `WHITELIST_MANAGER_ROLE`: Manage whitelist

### ERC20Factory Roles
- `FACTORY_ADMIN_ROLE`: Manage factory settings
- `TOKEN_CREATOR_ROLE`: Create new tokens
- `FACTORY_MINTER_ROLE`: Mint tokens
- `RATIO_MANAGER_ROLE`: Manage token ratios

### TokenSwap Roles
- `ADMIN_ROLE`: General admin operations
- `PAUSER_ROLE`: Pause/unpause swap functionality
- `FEE_MANAGER_ROLE`: Manage fee settings

## Usage Examples

### Creating a New Token

```solidity
// With TOKEN_CREATOR_ROLE
erc20Factory.createToken(
    "Asset Token",      // Token name
    "ASTKN",           // Token symbol
    adminAddress,      // Token owner
    100               // 100 tokens per 1 stablecoin
);