"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ERC20FactoryABI = void 0;
exports.ERC20FactoryABI = [
    // Access control functions
    "function hasRole(bytes32 role, address account) view returns (bool)",
    "function getRoleAdmin(bytes32 role) view returns (bytes32)",
    "function grantRole(bytes32 role, address account)",
    "function revokeRole(bytes32 role, address account)",
    "function renounceRole(bytes32 role, address callerConfirmation)",
    // Factory specific functions
    "function stableCoin() view returns (address)",
    "function tokenRatios(address token) view returns (uint256)",
    "function isTokenCreatedByFactory(address) view returns (bool)",
    "function allCreatedTokens(uint256) view returns (address)",
    "function createToken(string memory name, string memory symbol, address stableCoinAddress, address swapperAddress, address tokenOwner, uint256 tokensPerStableCoin) returns (address)",
    "function mintToken(address tokenAddress, address to, uint256 amount)",
    "function getAllTokenAddresses() view returns (address[] memory)",
    // Role constants
    "function FACTORY_ADMIN_ROLE() view returns (bytes32)",
    "function TOKEN_CREATOR_ROLE() view returns (bytes32)",
    "function FACTORY_MINTER_ROLE() view returns (bytes32)",
    "function RATIO_MANAGER_ROLE() view returns (bytes32)",
    // Events
    "event TokenCreated(address indexed creator, address indexed tokenAddress, string name, string symbol, address tokenOwner)",
    "event TokenMinted(address indexed tokenAddress, address indexed to, uint256 amount)",
    "event TokenRatioSet(address indexed tokenAddress, uint256 tokensPerStableCoin)",
    "event StableCoinAddressSet(address stableCoinAddress)",
    "event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)",
    "event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)",
];
