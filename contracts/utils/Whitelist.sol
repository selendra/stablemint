// SPDX-License-Identifier: MIT
// Version: 2.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "../interfaces/IWhitelist.sol";

/**
 * @title Whitelist
 * @dev Manages a whitelist of addresses
 * @notice Optimized for gas usage and integration with StableCoin
 */
contract Whitelist is IWhitelist, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant WHITELISTER_ROLE = keccak256("WHITELISTER_ROLE");
    bytes32 public constant CONTRACT_ROLE = keccak256("CONTRACT_ROLE");
    
    // Using a more gas-efficient mapping structure
    mapping(address => bool) private _whitelisted;
    
    // Global switch for whitelisting
    bool public whitelistingEnabled;
    
    // Batch operation size limit to prevent gas limit issues
    uint256 public constant MAX_BATCH_SIZE = 200;
    
    // Events
    event WhitelistUpdated(address indexed account, bool status);
    event WhitelistingToggled(bool enabled);
    event WhitelisterAdded(address indexed account);
    event WhitelisterRemoved(address indexed account);
    event ContractAuthorized(address indexed contractAddress);

    // Errors
    error ZeroAddress();
    error BatchTooLarge(uint256 size, uint256 maxSize);
    error NotAuthorized();
    
    /**
     * @dev Constructor to setup initial roles and settings
     * @param enableWhitelisting Whether to enable whitelisting at deploy time
     */
    constructor(bool enableWhitelisting) {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(WHITELISTER_ROLE, msg.sender);
        
        // Add deployer to whitelist
        _whitelisted[msg.sender] = true;
        emit WhitelistUpdated(msg.sender, true);
        
        // Set initial whitelisting state
        whitelistingEnabled = enableWhitelisting;
        if (enableWhitelisting) {
            emit WhitelistingToggled(true);
        }
    }
    
    /**
     * @dev Enable or disable whitelisting
     * @param enabled Whether to enable whitelisting
     */
    function toggleWhitelisting(bool enabled) external onlyRole(ADMIN_ROLE) {
        whitelistingEnabled = enabled;
        emit WhitelistingToggled(enabled);
    }
    
    /**
     * @dev Authorize a contract to access whitelist functions
     * @param contractAddress Contract to authorize
     */
    function authorizeContract(address contractAddress) external onlyRole(ADMIN_ROLE) {
        if (contractAddress == address(0)) revert ZeroAddress();
        _grantRole(CONTRACT_ROLE, contractAddress);
        emit ContractAuthorized(contractAddress);
    }
    
    /**
     * @dev Add an address as a whitelister
     * @param account Address to add as whitelister
     */
    function addWhitelister(address account) external onlyRole(ADMIN_ROLE) {
        if (account == address(0)) revert ZeroAddress();
        _grantRole(WHITELISTER_ROLE, account);
        emit WhitelisterAdded(account);
    }
    
    /**
     * @dev Remove an address as a whitelister
     * @param account Address to remove as whitelister
     */
    function removeWhitelister(address account) external onlyRole(ADMIN_ROLE) {
        _revokeRole(WHITELISTER_ROLE, account);
        emit WhitelisterRemoved(account);
    }
    
    /**
     * @dev Set whitelist status for an address
     * @param account Address to update
     * @param status Whitelist status to set
     */
    function setWhitelisted(address account, bool status) external onlyRole(WHITELISTER_ROLE) {
        _setWhitelisted(account, status);
    }
    
    /**
     * @dev Set whitelist status for multiple addresses at once
     * @param accounts Addresses to update
     * @param status Whitelist status to set for all addresses
     */
    function batchSetWhitelisted(address[] calldata accounts, bool status) external onlyRole(WHITELISTER_ROLE) {
        uint256 length = accounts.length;
        if (length > MAX_BATCH_SIZE) revert BatchTooLarge(length, MAX_BATCH_SIZE);
        
        for (uint256 i = 0; i < length;) {
            _setWhitelisted(accounts[i], status);
            unchecked { ++i; }
        }
    }

    /**
     * @dev Check if an address is whitelisted (public facing)
     * @param account Address to check
     * @return bool Whether the address is whitelisted
     */
    function isWhitelisted(address account) external view returns (bool) {
        if (!whitelistingEnabled) {
            return true; // All accounts pass when whitelisting is disabled
        }
        return _whitelisted[account];
    }
    
    /**
     * @dev Check if an address is whitelisted (for contract calls)
     * @param account Address to check
     * @return bool Whether the address is whitelisted
     */
    function checkWhitelist(address account) external view returns (bool) {
        // For contract calls - returns true if whitelist is disabled or account is whitelisted
        if (!whitelistingEnabled) {
            return true;
        }
        return _whitelisted[account];
    }
    
    /**
     * @dev Internal function to set whitelist status and emit event
     * @param account Address to update
     * @param status Whitelist status to set
     */
    function _setWhitelisted(address account, bool status) internal {
        _whitelisted[account] = status;
        emit WhitelistUpdated(account, status);
    }
}