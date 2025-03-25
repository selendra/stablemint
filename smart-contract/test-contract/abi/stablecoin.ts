export const StableCoinABI = [
  // ERC20 functions
  "function name() view returns (string)",
  "function symbol() view returns (string)",
  "function decimals() view returns (uint8)",
  "function totalSupply() view returns (uint256)",
  "function balanceOf(address account) view returns (uint256)",
  "function transfer(address to, uint256 amount) returns (bool)",
  "function allowance(address owner, address spender) view returns (uint256)",
  "function approve(address spender, uint256 amount) returns (bool)",
  "function transferFrom(address from, address to, uint256 amount) returns (bool)",

  // Access control functions
  "function hasRole(bytes32 role, address account) view returns (bool)",
  "function getRoleAdmin(bytes32 role) view returns (bytes32)",
  "function grantRole(bytes32 role, address account)",
  "function revokeRole(bytes32 role, address account)",
  "function renounceRole(bytes32 role, address callerConfirmation)",

  // StableCoin specific functions
  "function whitelisted(address account) view returns (bool)",
  "function addToWhitelist(address account)",
  "function removeFromWhitelist(address account)",
  "function batchAddToWhitelist(address[] calldata accounts)",
  "function mint(address to, uint256 amount)",
  "function burn(address from, uint256 amount)",
  "function withdraw(uint256 amount, address withdrawer, bytes32 data)",

  // Role constants
  "function PAUSER_ROLE() view returns (bytes32)",
  "function MINTER_ROLE() view returns (bytes32)",
  "function BURNER_ROLE() view returns (bytes32)",
  "function ADMIN_ROLE() view returns (bytes32)",
  "function WHITELIST_MANAGER_ROLE() view returns (bytes32)",

  // Events
  "event Transfer(address indexed from, address indexed to, uint256 value)",
  "event Approval(address indexed owner, address indexed spender, uint256 value)",
  "event Whitelisted(address indexed account, bool isWhitelisted)",
  "event withdrawEvent(uint256 amount, address withdrawer, bytes32 data)",
  "event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)",
  "event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)",
];
