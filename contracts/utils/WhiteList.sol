// SPDX-License-Identifier: MIT
// Version: 1.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";

contract Whitelist is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant WHITELISTER_ROLE = keccak256("WHITELISTER_ROLE");
    bytes32 public constant CONTRACT_ROLE = keccak256("CONTRACT_ROLE");
    
    mapping(address => bool) private _whitelisted;
    bool public whitelistingEnabled;
    
    event WhitelistUpdated(address indexed account, bool status);
    event WhitelistingToggled(bool enabled);
    event WhitelisterAdded(address indexed account);
    event WhitelisterRemoved(address indexed account);
    event ContractAuthorized(address indexed contractAddress);

    error ZeroAddress();
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(WHITELISTER_ROLE, msg.sender);
        
        // Add deployer to whitelist
        _whitelisted[msg.sender] = true;
        emit WhitelistUpdated(msg.sender, true);
    }
    
    function toggleWhitelisting(bool enabled) external onlyRole(ADMIN_ROLE) {
        whitelistingEnabled = enabled;
        emit WhitelistingToggled(enabled);
    }
    
    function authorizeContract(address contractAddress) external onlyRole(ADMIN_ROLE) {
        if (contractAddress == address(0)) revert ZeroAddress();
        _grantRole(CONTRACT_ROLE, contractAddress);
        emit ContractAuthorized(contractAddress);
    }
    
    function addWhitelister(address account) external onlyRole(ADMIN_ROLE) {
        if (account == address(0)) revert ZeroAddress();
        _grantRole(WHITELISTER_ROLE, account);
        emit WhitelisterAdded(account);
    }
    
    function removeWhitelister(address account) external onlyRole(ADMIN_ROLE) {
        _revokeRole(WHITELISTER_ROLE, account);
        emit WhitelisterRemoved(account);
    }
    
    function setWhitelisted(address account, bool status) external onlyRole(WHITELISTER_ROLE) {
        _whitelisted[account] = status;
        emit WhitelistUpdated(account, status);
    }
    
    function batchSetWhitelisted(address[] calldata accounts, bool status) external onlyRole(WHITELISTER_ROLE) {
        uint256 length = accounts.length;
        for (uint256 i = 0; i < length;) {
            _whitelisted[accounts[i]] = status;
            emit WhitelistUpdated(accounts[i], status);
            unchecked { ++i; }
        }
    }

    function isWhitelisted(address account) external view returns (bool) {
        if (!whitelistingEnabled) {
            return true; // All accounts pass when whitelisting is disabled
        }
        return _whitelisted[account];
    }
    
    function checkWhitelist(address account) external view returns (bool) {
        // For contract calls - returns true if whitelist is disabled or account is whitelisted
        if (!whitelistingEnabled) {
            return true;
        }
        return _whitelisted[account];
    }
}