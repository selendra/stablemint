export const STABLECOIN_ABI = [
    "function whitelisted(address account) external view returns (bool)",
    "function transfer(address to, uint256 amount) external returns (bool)",
    "function approve(address spender, uint256 amount) external returns (bool)",
    "function balanceOf(address account) external view returns (uint256)",
    "function allowance(address owner, address spender) external view returns (uint256)",
    "function paused() external view returns (bool)",
    "function decimals() external view returns (uint8)",
    "function enforceWhitelistForReceivers() external view returns (bool)",
    "event Transfer(address indexed from, address indexed to, uint256 value)",
    "event Approval(address indexed owner, address indexed spender, uint256 value)",
    "event Whitelisted(address indexed account, bool isWhitelisted)",
    "event WhitelistReceiverPolicyChanged(bool enforceForReceivers)",
    "event withdrawEvent(uint256 amount, address withdrawer, bytes32 data)"
];

export const STABLECOIN_ADMIN_ABI = [
    "function addToWhitelist(address account) external",
    "function removeFromWhitelist(address account) external",
    "function batchAddToWhitelist(address[] calldata accounts) external",
    "function setWhitelistReceiverPolicy(bool enforceForReceivers) external",
    "function mint(address to, uint256 amount) external",
    "function burn(uint256 amount) external",
    "function withdraw(uint256 amount, address withdrawer, bytes32 data) external",
    "function pause() external",
    "function unpause() external",
    "function whitelisted(address account) external view returns (bool)",
    "function enforceWhitelistForReceivers() external view returns (bool)",
    "function balanceOf(address account) external view returns (uint256)",
    "function paused() external view returns (bool)",
    "event Whitelisted(address indexed account, bool isWhitelisted)",
    "event WhitelistReceiverPolicyChanged(bool enforceForReceivers)",
    "event withdrawEvent(uint256 amount, address withdrawer, bytes32 data)"
];