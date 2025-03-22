export const ERC20FACTORY_ABI = [
    "function isTokenCreatedByFactory(address token) external view returns (bool)",
    "function tokenRatios(address token) external view returns (uint256)",
    "function stableCoin() external view returns (address)",
    "event TokenCreated(address indexed creator, address indexed tokenAddress, string name, string symbol, address tokenOwner)",
    "event TokenMinted(address indexed tokenAddress, address indexed to, uint256 amount)",
    "event TokenRatioSet(address indexed tokenAddress, uint256 tokensPerStableCoin)",
    "event StableCoinAddressSet(address stableCoinAddress)"
];

export const ERC20FACTORY_ADMIN_ABI = [
    "function createToken(string memory name, string memory symbol, address tokenOwner, uint256 tokensPerStableCoin) external returns (address)",
    "function mintToken(address tokenAddress, address to, uint256 amount) external",
    "function setStableCoinAddress(address _stableCoin) external",
    "function isTokenCreatedByFactory(address token) external view returns (bool)",
    "function tokenRatios(address token) external view returns (uint256)",
    "function stableCoin() external view returns (address)",
    "event TokenCreated(address indexed creator, address indexed tokenAddress, string name, string symbol, address tokenOwner)",
    "event TokenMinted(address indexed tokenAddress, address indexed to, uint256 amount)",
    "event TokenRatioSet(address indexed tokenAddress, uint256 tokensPerStableCoin)",
    "event StableCoinAddressSet(address stableCoinAddress)"
];
  