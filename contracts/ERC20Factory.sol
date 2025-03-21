// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./StableCoin.sol";
import "./utils/CustomERC20.sol";
import "./Swap.sol";

contract ERC20Factory is AccessControl, Pausable, ReentrancyGuard {
    bytes32 public constant FACTORY_ADMIN_ROLE = keccak256("FACTORY_ADMIN_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    StableCoin public stableCoin;
    TokenSwapper public swapper;
    
    mapping(address => bool) public isTokenCreatedByFactory;
    mapping(address => uint256) public tokenToStableRatio; // 1 token = x stablecoins
    mapping(address => address) public tokenCreator;
    
    event TokenCreated(address indexed tokenAddress, string name, string symbol, uint256 ratio, address creator);

    error InvalidAddress();
    error RatioMustBePositive();
    error StableCoinTransferFailed();
    
    constructor(address _stableCoinAddress) {
        if (_stableCoinAddress == address(0)) revert InvalidAddress();
        stableCoin = StableCoin(_stableCoinAddress);
        
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(FACTORY_ADMIN_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
    }
    
    function setSwapper(address _swapper) external onlyRole(FACTORY_ADMIN_ROLE) {
        if (_swapper == address(0)) revert InvalidAddress();
        swapper = TokenSwapper(_swapper);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }
    
    function createToken(
        string memory name,
        string memory symbol,
        uint256 initialSupply,
        uint256 stableCoinRatio
    ) external nonReentrant whenNotPaused returns (address) {
        if (stableCoinRatio == 0) revert RatioMustBePositive();
        
        // Calculate how many stablecoins are needed to back this token
        uint256 stableCoinAmount = initialSupply * 10**18 * stableCoinRatio;
        
        // Transfer stablecoins from the token creator to this contract
        bool transferSuccess = stableCoin.transferFrom(msg.sender, address(this), stableCoinAmount);
        if (!transferSuccess) revert StableCoinTransferFailed();
        
        // Create new token
        CustomERC20 newToken = new CustomERC20(
            name,
            symbol,
            initialSupply,
            msg.sender,
            address(stableCoin),
            address(swapper),
            stableCoinRatio
        );
        
        address tokenAddress = address(newToken);
        isTokenCreatedByFactory[tokenAddress] = true;
        tokenToStableRatio[tokenAddress] = stableCoinRatio;
        tokenCreator[tokenAddress] = msg.sender;
        
        // Register token with swapper if available
        if (address(swapper) != address(0)) {
            swapper.registerToken(tokenAddress, stableCoinRatio);
            // Transfer swapper role for minting/burning tokens
            newToken.grantMinterRole(address(swapper));
            
            // Approve stablecoin transfers to swapper
            stableCoin.approve(address(swapper), type(uint256).max);
        }
        
        emit TokenCreated(tokenAddress, name, symbol, stableCoinRatio, msg.sender);
        
        return tokenAddress;
    }
    
    function isFactoryToken(address tokenAddress) external view returns (bool) {
        return isTokenCreatedByFactory[tokenAddress];
    }
    
    function getTokenRatio(address tokenAddress) external view returns (uint256) {
        return tokenToStableRatio[tokenAddress];
    }
}