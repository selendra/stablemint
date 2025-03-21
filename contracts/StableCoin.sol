// SPDX-License-Identifier: MIT
// Version: 4.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./interfaces/IWhitelist.sol";
import "./interfaces/ITransferLimiter.sol";

/**
 * @title StableCoin
 * @dev ERC20 token with whitelist, transfer limits, and role-based access control
 * @notice Optimized to work with Whitelist and TransferLimiter contracts
 */
contract StableCoin is ERC20, AccessControl, Pausable, ReentrancyGuard {
    // Roles - using immutable for gas savings
    bytes32 private immutable MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 private immutable BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 private immutable PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    // External services
    IWhitelist public whitelistManager;
    ITransferLimiter public transferLimiter;
    
    // Configuration
    bool public limitChecksEnabled = true;
    bool public whitelistChecksEnabled = true;
    
    // Events
    event WhitelistManagerSet(address indexed managerAddress);
    event TransferLimiterSet(address indexed limiterAddress);
    event TokensMinted(address indexed to, uint256 amount);
    event TokensBurned(address indexed from, uint256 amount);
    event ConfigUpdated(bool limitChecksEnabled, bool whitelistChecksEnabled);

    // Errors
    error NotWhitelisted(address account);
    error ExceedsMaxAmount(uint256 amount, uint256 max);
    error CooldownNotElapsed(uint256 nextValidTime);
    error CannotRecoverSelf();
    error LimitExceeded();
    error ZeroAddress();
    error NotAuthorized();
    
    /**
     * @dev Constructor to initialize the token with name, symbol and initial supply
     * @param name Token name
     * @param symbol Token symbol
     * @param initialSupply Initial token supply (before decimals)
     * @param whitelistAddr Optional address of Whitelist contract (can be zero)
     * @param limiterAddr Optional address of TransferLimiter contract (can be zero)
     */
    constructor(
        string memory name, 
        string memory symbol, 
        uint256 initialSupply,
        address whitelistAddr,
        address limiterAddr
    ) ERC20(name, symbol) {
        // Set up roles
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(BURNER_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
        
        // Mint initial supply
        if (initialSupply > 0) {
            _mint(msg.sender, initialSupply * 10**decimals());
        }
        
        // Set up external contracts if provided
        if (whitelistAddr != address(0)) {
            _setWhitelistManager(whitelistAddr);
        }
        
        if (limiterAddr != address(0)) {
            _setTransferLimiter(limiterAddr);
        }
    }
    
    /**
     * @dev Set the whitelist manager contract
     * @param _whitelistManager Address of the whitelist manager contract
     */
    function setWhitelistManager(address _whitelistManager) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _setWhitelistManager(_whitelistManager);
    }
    
    /**
     * @dev Set the transfer limiter contract
     * @param _transferLimiter Address of the transfer limiter contract
     */
    function setTransferLimiter(address _transferLimiter) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _setTransferLimiter(_transferLimiter);
    }
    
    /**
     * @dev Update configuration settings
     * @param _limitChecksEnabled Whether transfer limit checks are enabled
     * @param _whitelistChecksEnabled Whether whitelist checks are enabled
     */
    function updateConfig(bool _limitChecksEnabled, bool _whitelistChecksEnabled) external onlyRole(DEFAULT_ADMIN_ROLE) {
        limitChecksEnabled = _limitChecksEnabled;
        whitelistChecksEnabled = _whitelistChecksEnabled;
        emit ConfigUpdated(_limitChecksEnabled, _whitelistChecksEnabled);
    }
    
    /**
     * @dev Pause token transfers
     */
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    /**
     * @dev Unpause token transfers
     */
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    /**
     * @dev Mint new tokens
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function mint(address to, uint256 amount) public onlyRole(MINTER_ROLE) {
        _checkWhitelist(to);
        _mint(to, amount);
        emit TokensMinted(to, amount);
    }
    
    /**
     * @dev Burn tokens from caller's account
     * @param amount Amount to burn
     */
    function burn(uint256 amount) public virtual onlyRole(BURNER_ROLE) {
        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount);
    }

    /**
     * @dev Burn tokens from another account (requires allowance)
     * @param account Account to burn from
     * @param amount Amount to burn
     */
    function burnFrom(address account, uint256 amount) public virtual onlyRole(BURNER_ROLE) {
        uint256 currentAllowance = allowance(account, msg.sender);
        if (currentAllowance < amount) revert ExceedsMaxAmount(amount, currentAllowance);
        
        unchecked {
            _approve(account, msg.sender, currentAllowance - amount);
        }
        _burn(account, amount);
        emit TokensBurned(account, amount);
    }
    
    /**
     * @dev Transfer tokens (with whitelist & limits checks)
     * @param to Recipient address
     * @param amount Amount to transfer
     * @return bool Success indicator
     */
    function transfer(address to, uint256 amount) public override whenNotPaused returns (bool) {
        _beforeTokenTransfer(msg.sender, to, amount);
        return super.transfer(to, amount);
    }
    
    /**
     * @dev Transfer tokens from another account (with whitelist & limits checks)
     * @param from Sender address
     * @param to Recipient address
     * @param amount Amount to transfer
     * @return bool Success indicator
     */
    function transferFrom(address from, address to, uint256 amount) public override whenNotPaused returns (bool) {
        _beforeTokenTransfer(from, to, amount);
        return super.transferFrom(from, to, amount);
    }
    
    /**
     * @dev Recover accidentally sent ERC20 tokens
     * @param tokenAddress Address of token to recover
     * @param amount Amount to recover
     */
    function recoverERC20(address tokenAddress, uint256 amount) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        if (tokenAddress == address(this)) revert CannotRecoverSelf();
        IERC20(tokenAddress).transfer(msg.sender, amount);
    }
    
    /**
     * @dev Performs pre-transfer checks (whitelist, limits, cooldown)
     * @param from Sender address
     * @param to Recipient address
     * @param amount Amount to transfer
     */
    function _beforeTokenTransfer(address from, address to, uint256 amount) internal {
        // Skip checks for minting and burning operations
        if (from == address(0) || to == address(0)) return;
        
        // Check whitelist if applicable
        if (whitelistChecksEnabled) {
            _checkWhitelist(from);
            _checkWhitelist(to);
            _checkWhitelist(msg.sender);
        }
        
        // Check transfer limits if applicable
        if (limitChecksEnabled) {
            _checkTransferLimit(from, amount);
            _enforceCooldown(from);
            _recordTransfer(from, amount);
        }
    }
    
    /**
     * @dev Internal function to set the whitelist manager
     * @param _whitelistManager Address of the whitelist manager contract
     */
    function _setWhitelistManager(address _whitelistManager) internal {
        if (_whitelistManager == address(0)) revert ZeroAddress();
        whitelistManager = IWhitelist(_whitelistManager);
        
        // Try to authorize the token in the whitelist
        try IWhitelist(_whitelistManager).authorizeContract(address(this)) {} catch {}
        
        emit WhitelistManagerSet(_whitelistManager);
    }
    
    /**
     * @dev Internal function to set the transfer limiter
     * @param _transferLimiter Address of the transfer limiter contract
     */
    function _setTransferLimiter(address _transferLimiter) internal {
        if (_transferLimiter == address(0)) revert ZeroAddress();
        transferLimiter = ITransferLimiter(_transferLimiter);
        
        // Try to authorize the token in the limiter
        try ITransferLimiter(_transferLimiter).authorizeContract(address(this)) {} catch {}
        
        emit TransferLimiterSet(_transferLimiter);
    }
    
    /**
     * @dev Check if an account is whitelisted
     * @param account Address to check
     */
    function _checkWhitelist(address account) internal view {
        if (!whitelistChecksEnabled || address(whitelistManager) == address(0)) {
            return; // Skip check if not enabled or no whitelist manager
        }
        
        if (!whitelistManager.checkWhitelist(account)) {
            revert NotWhitelisted(account);
        }
    }
    
    /**
     * @dev Check if a transfer exceeds limits
     * @param sender Sender address
     * @param amount Amount to transfer
     */
    function _checkTransferLimit(address sender, uint256 amount) internal view {
        if (!limitChecksEnabled || address(transferLimiter) == address(0)) {
            return; // Skip check if not enabled or no limiter
        }
        
        if (!transferLimiter.checkTransferLimit(address(this), sender, amount)) {
            revert LimitExceeded();
        }
    }
    
    /**
     * @dev Enforce cooldown period between transfers
     * @param sender Sender address
     */
    function _enforceCooldown(address sender) internal {
        if (!limitChecksEnabled || address(transferLimiter) == address(0)) {
            return; // Skip check if not enabled or no limiter
        }
        
        transferLimiter.enforceCooldown(address(this), sender);
    }
    
    /**
     * @dev Record a transfer for limit tracking
     * @param sender Sender address
     * @param amount Amount transferred
     */
    function _recordTransfer(address sender, uint256 amount) internal {
        if (!limitChecksEnabled || address(transferLimiter) == address(0)) {
            return; // Skip if not enabled or no limiter
        }
        
        transferLimiter.recordTransfer(address(this), sender, amount);
    }
}