export const TOKENSWAP_ABI = [
    "function swapStableCoinToToken(address token, uint256 stableCoinAmount) external",
    "function swapTokenToStableCoin(address token, uint256 tokenAmount) external",
    "function stableCoin() external view returns (address)",
    "function tokenFactory() external view returns (address)",
    "function feePercentage() external view returns (uint256)",
    "function feeCollector() external view returns (address)",
    "function paused() external view returns (bool)",
    "event StableCoinToToken(address indexed user, address indexed token, uint256 stableCoinAmount, uint256 tokenAmount, uint256 feeAmount)",
    "event TokenToStableCoin(address indexed user, address indexed token, uint256 tokenAmount, uint256 stableCoinAmount, uint256 feeAmount)"
];

export const TOKENSWAP_ADMIN_ABI = [
    "function setFeePercentage(uint256 _feePercentage) external",
    "function setFeeCollector(address _feeCollector) external",
    "function pause() external",
    "function unpause() external",
    "function emergencyWithdraw(address token, uint256 amount, address to) external",
    "function stableCoin() external view returns (address)",
    "function tokenFactory() external view returns (address)",
    "function feePercentage() external view returns (uint256)",
    "function feeCollector() external view returns (address)",
    "function paused() external view returns (bool)",
    "event FeeUpdated(uint256 newFeePercentage)",
    "event FeeCollectorUpdated(address newFeeCollector)"
];
  