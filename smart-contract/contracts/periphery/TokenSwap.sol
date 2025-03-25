// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "../interfaces/IStableCoin.sol";
import "../interfaces/IERC20Factory.sol";

contract TokenSwap is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using SafeERC20 for IStableCoin;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    IStableCoin public stableCoin;
    IERC20Factory public tokenFactory;

    event StableCoinToToken(
        address indexed user,
        address indexed token,
        uint256 stableCoinAmount,
        uint256 tokenAmount
    );

    event TokenToStableCoin(
        address indexed user,
        address indexed token,
        uint256 tokenAmount,
        uint256 stableCoinAmount
    );

    constructor(address _stableCoin, address _tokenFactory, address _admin) {
        require(_stableCoin != address(0), "StableCoin cannot be zero address");
        require(
            _tokenFactory != address(0),
            "TokenFactory cannot be zero address"
        );
        require(_admin != address(0), "Admin cannot be zero address");

        stableCoin = IStableCoin(_stableCoin);
        tokenFactory = IERC20Factory(_tokenFactory);

        _grantRole(DEFAULT_ADMIN_ROLE, _admin);
        _grantRole(ADMIN_ROLE, _admin);
        _grantRole(PAUSER_ROLE, _admin);
    }

    /**
     * @dev Swap StableCoin for Token
     * @param token The token address to receive
     * @param stableCoinAmount The amount of StableCoin to swap
     */
    function swapStableCoinToToken(
        address token,
        uint256 stableCoinAmount
    ) external nonReentrant {
        require(stableCoinAmount > 0, "Amount must be greater than zero");
        require(
            tokenFactory.isTokenCreatedByFactory(token),
            "Token not supported"
        );

        // Check if user is whitelisted
        require(stableCoin.whitelisted(msg.sender), "User not whitelisted");

        // Check user's stablecoin balance
        require(
            stableCoin.balanceOf(msg.sender) >= stableCoinAmount,
            "Insufficient StableCoin balance"
        );

        // Calculate token amount based on ratio
        uint256 tokenRatio = tokenFactory.tokenRatios(token);
        require(tokenRatio > 0, "Token ratio not set");

        uint256 tokenAmount = stableCoinAmount * tokenRatio;

        // Check for potential overflow (although Solidity 0.8+ has built-in checks)
        require(
            tokenAmount / tokenRatio == stableCoinAmount,
            "Multiplication overflow"
        );

        // Transfer StableCoin from user to token contract
        stableCoin.safeTransferFrom(msg.sender, token, stableCoinAmount);

        // Mint tokens to user via factory
        tokenFactory.mintToken(token, msg.sender, tokenAmount);

        emit StableCoinToToken(
            msg.sender,
            token,
            stableCoinAmount,
            tokenAmount
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
    ) external nonReentrant {
        require(tokenAmount > 0, "Amount must be greater than zero");
        require(
            tokenFactory.isTokenCreatedByFactory(token),
            "Token not supported"
        );

        // Check if user is whitelisted
        require(stableCoin.whitelisted(msg.sender), "User not whitelisted");

        // Check user's token balance
        require(
            IERC20(token).balanceOf(msg.sender) >= tokenAmount,
            "Insufficient token balance"
        );

        // Calculate stablecoin amount based on ratio
        uint256 tokenRatio = tokenFactory.tokenRatios(token);
        require(tokenRatio > 0, "Token ratio not set");

        // Ensure the token amount is sufficient for at least 1 stablecoin
        require(
            tokenAmount >= tokenRatio,
            "Token amount too small for conversion"
        );
        uint256 stableCoinAmount = tokenAmount / tokenRatio;

        // Check token contract has enough StableCoin balance
        require(
            stableCoin.balanceOf(token) >= stableCoinAmount,
            "Insufficient stablecoin balance in token contract"
        );

        // Transfer tokens from user to this contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), tokenAmount);

        // Burn the tokens
        IERC20Burnable(token).burn(tokenAmount);

        // // Transfer StableCoin from token contract to user
        // // Need to have the token contract approve this contract first
        stableCoin.safeTransferFrom(token, msg.sender, stableCoinAmount);

        emit TokenToStableCoin(
            msg.sender,
            token,
            tokenAmount,
            stableCoinAmount
        );
    }
}
