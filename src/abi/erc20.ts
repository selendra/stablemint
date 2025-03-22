export const ERC20TOKEN_ABI = [
    "function transfer(address to, uint256 amount) external returns (bool)",
    "function approve(address spender, uint256 amount) external returns (bool)",
    "function balanceOf(address account) external view returns (uint256)",
    "function allowance(address owner, address spender) external view returns (uint256)",
    "function factory() external view returns (address)",
    "function paused() external view returns (bool)",
    "function decimals() external view returns (uint8)",
    "event Transfer(address indexed from, address indexed to, uint256 value)",
    "event Approval(address indexed owner, address indexed spender, uint256 value)",
    "event RoleAdminChanged(bytes32 indexed role, address indexed account, address indexed caller)"
];
  

export const ERC20TOKEN_ADMIN_ABI = [
    "function pause() external",
    "function unpause() external",
    "function factory() external view returns (address)",
    "function paused() external view returns (bool)",
    "function balanceOf(address account) external view returns (uint256)"
];