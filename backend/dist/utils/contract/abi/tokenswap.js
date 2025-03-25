"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TokenSwapABI = void 0;
exports.TokenSwapABI = [
    // Access control functions
    "function hasRole(bytes32 role, address account) view returns (bool)",
    "function getRoleAdmin(bytes32 role) view returns (bytes32)",
    "function grantRole(bytes32 role, address account)",
    "function revokeRole(bytes32 role, address account)",
    "function renounceRole(bytes32 role, address callerConfirmation)",
    // TokenSwap specific functions
    "function stableCoin() view returns (address)",
    "function tokenFactory() view returns (address)",
    "function swapStableCoinToToken(address token, uint256 stableCoinAmount)",
    "function swapTokenToStableCoin(address token, uint256 tokenAmount)",
    // Role constants
    "function ADMIN_ROLE() view returns (bytes32)",
    "function PAUSER_ROLE() view returns (bytes32)",
    // Events
    "event StableCoinToToken(address indexed user, address indexed token, uint256 stableCoinAmount, uint256 tokenAmount)",
    "event TokenToStableCoin(address indexed user, address indexed token, uint256 tokenAmount, uint256 stableCoinAmount)",
    "event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)",
    "event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)",
];
