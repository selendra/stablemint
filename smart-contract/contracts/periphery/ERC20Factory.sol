// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../StableCoin.sol";
import "../ERC20Token.sol";

contract ERC20Factory is AccessControl, ReentrancyGuard {
    bytes32 public constant FACTORY_ADMIN_ROLE =
        keccak256("FACTORY_ADMIN_ROLE");
    bytes32 public constant TOKEN_CREATOR_ROLE =
        keccak256("TOKEN_CREATOR_ROLE");
    bytes32 public constant FACTORY_MINTER_ROLE =
        keccak256("FACTORY_MINTER_ROLE");
    bytes32 public constant RATIO_MANAGER_ROLE =
        keccak256("RATIO_MANAGER_ROLE");

    // Collateralization information
    StableCoin public stableCoin;
    mapping(address => uint256) public tokenRatios; // token address => tokens minted per 1 StableCoin

    // Tracking created tokens
    mapping(address => bool) public isTokenCreatedByFactory;

    // Array to keep track of all created token addresses
    address[] public allCreatedTokens;

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
        require(
            initialOwner != address(0),
            "Factory owner cannot be zero address"
        );
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
        address stableCoinAddress,
        address swapperAddress,
        address tokenOwner,
        uint256 tokensPerStableCoin
    ) external onlyRole(TOKEN_CREATOR_ROLE) nonReentrant returns (address) {
        // More specific error messages
        if (bytes(name).length == 0) revert("Token name cannot be empty");
        if (bytes(symbol).length == 0) revert("Token symbol cannot be empty");
        if (tokenOwner == address(0))
            revert("Cannot grant role to zero address");
        if (tokensPerStableCoin == 0)
            revert("Tokens per StableCoin must be greater than zero");

        // Create token with try/catch to better handle errors
        ERC20Token newToken;
        try
            new ERC20Token(
                name,
                symbol,
                tokenOwner,
                stableCoinAddress,
                swapperAddress
            )
        returns (ERC20Token _token) {
            newToken = _token;
        } catch Error(string memory reason) {
            revert(
                string(abi.encodePacked("Failed to create token: ", reason))
            );
        } catch {
            revert("Failed to create token: unknown error");
        }

        // Register the token
        address tokenAddress = address(newToken);
        isTokenCreatedByFactory[tokenAddress] = true;

        // Add the token address to our array
        allCreatedTokens.push(tokenAddress);

        // Set the initial ratio for this token
        tokenRatios[tokenAddress] = tokensPerStableCoin;
        emit TokenRatioSet(tokenAddress, tokensPerStableCoin);

        // Emit event (this is critical to fix your issue)
        emit TokenCreated(msg.sender, tokenAddress, name, symbol, tokenOwner);

        return tokenAddress;
    }

    function mintToken(
        address tokenAddress,
        address to,
        uint256 amount
    ) external onlyRole(FACTORY_MINTER_ROLE) nonReentrant {
        require(
            isTokenCreatedByFactory[tokenAddress],
            "Token not created by this factory"
        );

        uint256 tokensPerStableCoin = tokenRatios[tokenAddress];
        require(tokensPerStableCoin > 0, "Token ratio not set");

        ERC20Token token = ERC20Token(tokenAddress);

        // Get the current total supply of the token
        uint256 currentSupply = token.totalSupply();

        // Get the actual StableCoin balance held by the token address
        uint256 stableCoinBalance = stableCoin.balanceOf(tokenAddress);

        // Calculate maximum tokens that can be minted based on available StableCoin balance
        uint256 maxTokensAllowed = stableCoinBalance * tokensPerStableCoin;

        // Ensure the total supply after minting doesn't exceed the allowed amount
        require(
            currentSupply + amount <= maxTokensAllowed,
            "Total supply would exceed available StableCoin collateral"
        );

        token.mint(to, amount);

        emit TokenMinted(tokenAddress, to, amount);
    }

    // Function to get all created token addresses
    function getAllTokenAddresses() public view returns (address[] memory) {
        return allCreatedTokens;
    }
}
