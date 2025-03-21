// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "../interfaces/IStableCoin.sol";
import "../interfaces/IERC20Factory.sol";

contract TokenSwap is AccessControl, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;
    using SafeERC20 for IStableCoin;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    bytes32 public constant FEE_MANAGER_ROLE = keccak256("FEE_MANAGER_ROLE");

    IStableCoin public stableCoin;
    IERC20Factory public tokenFactory;
    
    uint256 public feePercentage; // In basis points (1/100 of a percent, e.g., 25 = 0.25%)
    address public feeCollector;
    
    uint256 public constant MAX_FEE = 500; // 5% maximum fee
    uint256 public constant BASIS_POINTS = 10000; // 100% in basis points

    event StableCoinToToken(
        address indexed user, 
        address indexed token, 
        uint256 stableCoinAmount, 
        uint256 tokenAmount,
        uint256 feeAmount
    );
    
    event TokenToStableCoin(
        address indexed user, 
        address indexed token, 
        uint256 tokenAmount, 
        uint256 stableCoinAmount,
        uint256 feeAmount
    );
    
    event FeeUpdated(uint256 newFeePercentage);
    event FeeCollectorUpdated(address newFeeCollector);

    constructor(
        address _stableCoin,
        address _tokenFactory,
        address _admin,
        address _feeCollector,
        uint256 _initialFeePercentage
    ) {
        require(_stableCoin != address(0), "StableCoin cannot be zero address");
        require(_tokenFactory != address(0), "TokenFactory cannot be zero address");
        require(_admin != address(0), "Admin cannot be zero address");
        require(_feeCollector != address(0), "Fee collector cannot be zero address");
        require(_initialFeePercentage <= MAX_FEE, "Fee too high");
        
        stableCoin = IStableCoin(_stableCoin);
        tokenFactory = IERC20Factory(_tokenFactory);
        feeCollector = _feeCollector;
        feePercentage = _initialFeePercentage;
        
        _grantRole(DEFAULT_ADMIN_ROLE, _admin);
        _grantRole(ADMIN_ROLE, _admin);
        _grantRole(PAUSER_ROLE, _admin);
        _grantRole(FEE_MANAGER_ROLE, _admin);
    }
    
    /**
     * @dev Swap StableCoin for Token
     * @param token The token address to receive
     * @param stableCoinAmount The amount of StableCoin to swap
     */
    function swapStableCoinToToken(
        address token,
        uint256 stableCoinAmount
    ) external whenNotPaused nonReentrant {
        require(stableCoinAmount > 0, "Amount must be greater than zero");
        require(tokenFactory.isTokenCreatedByFactory(token), "Token not supported");
        
        // Check if user is whitelisted
        require(stableCoin.whitelisted(msg.sender), "User not whitelisted");
        
        // Calculate token amount based on ratio
        uint256 tokenRatio = tokenFactory.tokenRatios(token);
        require(tokenRatio > 0, "Token ratio not set");
        
        uint256 tokenAmount = stableCoinAmount * tokenRatio;
        
        // Calculate fee
        uint256 feeAmount = 0;
        if (feePercentage > 0) {
            feeAmount = (stableCoinAmount * feePercentage) / BASIS_POINTS;
            require(feeAmount < stableCoinAmount, "Fee exceeds amount");
        }
        
        // Transfer StableCoin from user to this contract
        stableCoin.safeTransferFrom(msg.sender, address(this), stableCoinAmount);
        
        // Send fee to collector
        if (feeAmount > 0) {
            stableCoin.safeTransfer(feeCollector, feeAmount);
        }
        
        // Transfer tokens to user
        IERC20(token).safeTransfer(msg.sender, tokenAmount);
        
        emit StableCoinToToken(
            msg.sender, 
            token, 
            stableCoinAmount, 
            tokenAmount, 
            feeAmount
        );
    }
    
    /**
     * @dev Swap Token for StableCoin
     * @param token The token address to swap
     * @param tokenAmount The amount of tokens to swap
     */
    function swapTokenToStableCoin(
        address token,
        uint256 tokenAmount
    ) external whenNotPaused nonReentrant {
        require(tokenAmount > 0, "Amount must be greater than zero");
        require(tokenFactory.isTokenCreatedByFactory(token), "Token not supported");
        
        // Check if user is whitelisted
        require(stableCoin.whitelisted(msg.sender), "User not whitelisted");
        
        // Calculate stablecoin amount based on ratio
        uint256 tokenRatio = tokenFactory.tokenRatios(token);
        require(tokenRatio > 0, "Token ratio not set");
        
        uint256 stableCoinAmount = tokenAmount / tokenRatio;
        require(stableCoinAmount > 0, "StableCoin amount too small");
        
        // Calculate fee
        uint256 feeAmount = 0;
        if (feePercentage > 0) {
            feeAmount = (stableCoinAmount * feePercentage) / BASIS_POINTS;
            require(feeAmount < stableCoinAmount, "Fee exceeds amount");
        }
        
        uint256 stableCoinToTransfer = stableCoinAmount - feeAmount;
        
        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);
        
        // Transfer StableCoin to user
        stableCoin.safeTransfer(msg.sender, stableCoinToTransfer);
        
        // Send fee to collector
        if (feeAmount > 0) {
            stableCoin.safeTransfer(feeCollector, feeAmount);
        }
        
        emit TokenToStableCoin(
            msg.sender, 
            token, 
            tokenAmount, 
            stableCoinAmount, 
            feeAmount
        );
    }
    
    function setFeePercentage(uint256 _feePercentage) external onlyRole(FEE_MANAGER_ROLE) {
        require(_feePercentage <= MAX_FEE, "Fee too high");
        feePercentage = _feePercentage;
        emit FeeUpdated(_feePercentage);
    }
    
    function setFeeCollector(address _feeCollector) external onlyRole(ADMIN_ROLE) {
        require(_feeCollector != address(0), "Fee collector cannot be zero address");
        feeCollector = _feeCollector;
        emit FeeCollectorUpdated(_feeCollector);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function emergencyWithdraw(
        address token,
        uint256 amount,
        address to
    ) external onlyRole(ADMIN_ROLE) {
        require(to != address(0), "Cannot withdraw to zero address");
        IERC20(token).safeTransfer(to, amount);
    }
}