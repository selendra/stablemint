export const ROLE_ADMIN_ABI = [
    "function grantRole(bytes32 role, address account) external",
    "function revokeRole(bytes32 role, address account) external",
    "function hasRole(bytes32 role, address account) external view returns (bool)",
    "event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)",
    "event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)"
];