export { ERC20TOKEN_ABI, ERC20TOKEN_ADMIN_ABI } from "./erc20";
export { ERC20FACTORY_ABI, ERC20FACTORY_ADMIN_ABI } from "./factory";
export { STABLECOIN_ABI, STABLECOIN_ADMIN_ABI } from "./stableCoin";
export { TOKENSWAP_ABI, TOKENSWAP_ADMIN_ABI } from "./swap";

export const ROLE_ADMIN_ABI = [
  "function grantRole(bytes32 role, address account) external",
  "function revokeRole(bytes32 role, address account) external",
  "function hasRole(bytes32 role, address account) external view returns (bool)",
  "event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)",
  "event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)",
];

export const PAUSE_ADMIN_ABI = [
  "function pause() external",
  "function unpause() external",
  "function paused() external view returns (bool)",
];

export const BALANCE_ABI = [
  "function balanceOf(address account) external view returns (uint256)",
];
