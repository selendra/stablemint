// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./StableCoin.sol";
import "./utils/CustomERC20.sol";

interface IFactory {
    function isFactoryToken(address tokenAddress) external view returns (bool);
    function getTokenRatio(address tokenAddress) external view returns (uint256);
    function tokenCreator(address tokenAddress) external view returns (address);
}

contract TokenSwapper is AccessControl, Pausable, ReentrancyGuard {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    StableCoin public stableCoin;
    IFactory public factory;
    
    // Swap settings per token
    mapping(address => bool) public tokenSwapEnabled;
    
    // Fee settings
    uint256 public swapFeePercent = 50; // 0.5% in basis points (1% = 100)
    address public feeCollector;
    
    // Events
    event TokenRegistered(address indexed tokenAddress, uint256 ratio);
    event TokenSwapStatusChanged(address indexed tokenAddress, bool enabled);
    event SwapFeeUpdated(uint256 newFeePercent);
    event FeeCollectorUpdated(address indexed collector);
    event SwapExecuted(address indexed tokenAddress, address indexed user, uint256 amount, bool toStable);

    error NotFactoryToken();
    error SwapDisabled();
    error ZeroAmount();
    error FeeExceedsMaximum();
    error InvalidAddress();
    error StableCoinTransferFailed();
    error AmountNotDivisible();
    error NotAuthorized();
        
    constructor(address _stableCoinAddress, address _factoryAddress) {
        if (_stableCoinAddress == address(0) || _factoryAddress == address(0)) {
            revert InvalidAddress();
        }
        
        stableCoin = StableCoin(_stableCoinAddress);
        factory = IFactory(_factoryAddress);
        
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, _factoryAddress);
        
        feeCollector = msg.sender;
    }
    
    function registerToken(address tokenAddress, uint256 ratio) external onlyRole(ADMIN_ROLE) {
        tokenSwapEnabled[tokenAddress] = true;
        emit TokenRegistered(tokenAddress, ratio);
    }
    
    function setSwapFeePercent(uint256 fee) external onlyRole(ADMIN_ROLE) {
        if (fee > 500) revert FeeExceedsMaximum();
        swapFeePercent = fee;
        emit SwapFeeUpdated(fee);
    }
    
    function setFeeCollector(address collector) external onlyRole(ADMIN_ROLE) {
        if (collector == address(0)) revert InvalidAddress();
        feeCollector = collector;
        emit FeeCollectorUpdated(collector);
    }
    
    function setTokenSwapEnabled(address tokenAddress, bool enabled) external {
        if (!factory.isFactoryToken(tokenAddress)) revert NotFactoryToken();
        if (!(hasRole(ADMIN_ROLE, msg.sender) || factory.tokenCreator(tokenAddress) == msg.sender)) {
            revert NotAuthorized();
        }
        
        tokenSwapEnabled[tokenAddress] = enabled;
        emit TokenSwapStatusChanged(tokenAddress, enabled);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }
    
    function swapTokenForStable(address tokenAddress, uint256 tokenAmount) external nonReentrant whenNotPaused {
        if (!factory.isFactoryToken(tokenAddress)) revert NotFactoryToken();
        if (!tokenSwapEnabled[tokenAddress]) revert SwapDisabled();
        if (tokenAmount == 0) revert ZeroAmount();
        
        uint256 ratio = factory.getTokenRatio(tokenAddress);
        uint256 stableAmount = tokenAmount * ratio;
        
        // Calculate fee
        uint256 feeAmount = (stableAmount * swapFeePercent) / 10000;
        uint256 netStableAmount = stableAmount - feeAmount;
        
        // Transfer tokens from user to this contract and burn them
        CustomERC20(tokenAddress).burnFrom(msg.sender, tokenAmount);
        
        // Transfer stablecoins to user and fees to collector
        if (feeAmount > 0) {
            bool feeSuccess = stableCoin.transfer(feeCollector, feeAmount);
            if (!feeSuccess) revert StableCoinTransferFailed();
        }
        
        bool transferSuccess = stableCoin.transfer(msg.sender, netStableAmount);
        if (!transferSuccess) revert StableCoinTransferFailed();
        
        emit SwapExecuted(tokenAddress, msg.sender, tokenAmount, true);
    }
    
    function swapStableForToken(address tokenAddress, uint256 stableAmount) external nonReentrant whenNotPaused {
        if (!factory.isFactoryToken(tokenAddress)) revert NotFactoryToken();
        if (!tokenSwapEnabled[tokenAddress]) revert SwapDisabled();
        if (stableAmount == 0) revert ZeroAmount();
        
        uint256 ratio = factory.getTokenRatio(tokenAddress);
        
        // Calculate fee
        uint256 feeAmount = (stableAmount * swapFeePercent) / 10000;
        uint256 netStableAmount = stableAmount - feeAmount;
        
        if (netStableAmount % ratio != 0) revert AmountNotDivisible();
        uint256 tokenAmount = netStableAmount / ratio;
        
        // Transfer stablecoins from user to this contract
        bool transferSuccess = stableCoin.transferFrom(msg.sender, address(this), stableAmount);
        if (!transferSuccess) revert StableCoinTransferFailed();
        
        // Transfer fee to collector
        if (feeAmount > 0) {
            bool feeSuccess = stableCoin.transfer(feeCollector, feeAmount);
            if (!feeSuccess) revert StableCoinTransferFailed();
        }
        
        // Mint new tokens to user
        CustomERC20(tokenAddress).mint(msg.sender, tokenAmount);
        
        emit SwapExecuted(tokenAddress, msg.sender, tokenAmount, false);
    }
}