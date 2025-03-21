// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../StableCoin.sol";
import "../ERC20Token.sol";

contract ERC20Factory is AccessControl, ReentrancyGuard {
    bytes32 public constant FACTORY_ADMIN_ROLE = keccak256("FACTORY_ADMIN_ROLE");
    bytes32 public constant TOKEN_CREATOR_ROLE = keccak256("TOKEN_CREATOR_ROLE");
    bytes32 public constant FACTORY_MINTER_ROLE = keccak256("FACTORY_MINTER_ROLE");
    bytes32 public constant RATIO_MANAGER_ROLE = keccak256("RATIO_MANAGER_ROLE");
    
    // Collateralization information
    StableCoin public stableCoin;
    mapping(address => uint256) public tokenRatios; // token address => tokens minted per 1 StableCoin
    
    // Tracking created tokens
    mapping(address => bool) public isTokenCreatedByFactory;
    
    event TokenCreated(
        address indexed creator, 
        address indexed tokenAddress, 
        string name, 
        string symbol,
        address tokenOwner
    );
    
    event TokenMinted(
        address indexed tokenAddress,
        address indexed to,
        uint256 amount
    );
    
    event TokenRatioSet(
        address indexed tokenAddress,
        uint256 tokensPerStableCoin
    );
    
    event StableCoinAddressSet(address stableCoinAddress);
    
    constructor(address initialOwner, address _stableCoin) {
        require(initialOwner != address(0), "Factory owner cannot be zero address");
        require(_stableCoin != address(0), "StableCoin address cannot be zero");
        
        stableCoin = StableCoin(_stableCoin);
        
        _grantRole(DEFAULT_ADMIN_ROLE, initialOwner);
        _grantRole(FACTORY_ADMIN_ROLE, initialOwner);
        _grantRole(TOKEN_CREATOR_ROLE, initialOwner);
        _grantRole(FACTORY_MINTER_ROLE, initialOwner);
        _grantRole(RATIO_MANAGER_ROLE, initialOwner);
    }

    function createToken(
        string memory name,
        string memory symbol,
        address tokenOwner,
        uint256 tokensPerStableCoin
    ) external onlyRole(TOKEN_CREATOR_ROLE) nonReentrant returns (address) {
        require(bytes(name).length > 0, "Token name cannot be empty");
        require(bytes(symbol).length > 0, "Token symbol cannot be empty");
        require(tokenOwner != address(0), "Cannot grant role to zero address");
        require(tokensPerStableCoin > 0, "Tokens per StableCoin must be greater than zero");
        
        ERC20Token newToken = new ERC20Token(
            name,
            symbol,
            tokenOwner
        );
        
        // Register the token
        address tokenAddress = address(newToken);
        isTokenCreatedByFactory[tokenAddress] = true;
        
        // Set the initial ratio for this token
        tokenRatios[tokenAddress] = tokensPerStableCoin;
        emit TokenRatioSet(tokenAddress, tokensPerStableCoin);
        
        emit TokenCreated(
            msg.sender, 
            tokenAddress, 
            name, 
            symbol, 
            tokenOwner
        );
        
        return tokenAddress;
    }

    function mintToken(
        address tokenAddress, 
        address to, 
        uint256 amount
    ) external onlyRole(FACTORY_MINTER_ROLE) nonReentrant {
        require(isTokenCreatedByFactory[tokenAddress], "Token not created by this factory");
        
        uint256 tokensPerStableCoin = tokenRatios[tokenAddress];
        require(tokensPerStableCoin > 0, "Token ratio not set");
        
        uint256 requiredStableCoin = (amount + tokensPerStableCoin - 1) / tokensPerStableCoin;
        
        uint256 stableCoinBalance = stableCoin.balanceOf(msg.sender);
        require(stableCoinBalance >= requiredStableCoin, 
                "Insufficient StableCoin balance for minting");
        
        ERC20Token token = ERC20Token(tokenAddress);
        token.mint(to, amount);
        
        emit TokenMinted(tokenAddress, to, amount);
    }
    
    function setStableCoinAddress(address _stableCoin) 
        external 
        onlyRole(FACTORY_ADMIN_ROLE) 
    {
        require(_stableCoin != address(0), "StableCoin address cannot be zero");
        stableCoin = StableCoin(_stableCoin);
        emit StableCoinAddressSet(_stableCoin);
    }
}