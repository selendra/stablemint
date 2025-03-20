// SPDX-License-Identifier: MIT
// Version: 1.1.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";

contract TransferLimiter is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant CONTRACT_ROLE = keccak256("CONTRACT_ROLE");
    bytes32 public constant LIMIT_MANAGER_ROLE = keccak256("LIMIT_MANAGER_ROLE");
    
    // Token-wide settings
    mapping(address => uint256) public defaultMaxTransferAmount;
    mapping(address => uint256) public defaultTransferCooldown;
    
    // Per-user settings
    mapping(address => mapping(address => uint256)) public userMaxTransferAmount;
    mapping(address => mapping(address => uint256)) public userTransferCooldown;
    
    // Has custom limits
    mapping(address => mapping(address => bool)) public hasCustomLimits;
    mapping(address => mapping(address => bool)) public hasCustomCooldown;
    
    // Transfer tracking
    mapping(address => mapping(address => uint256)) public lastTransferTime;
    mapping(address => mapping(address => uint256)) public userDailyTotal;
    mapping(address => mapping(address => uint256)) public userDailyResetTime;
    
    // Exemptions
    mapping(address => mapping(address => bool)) public exemptFromLimits;
    
    // Events
    event DefaultMaxTransferUpdated(address indexed token, uint256 amount);
    event DefaultCooldownUpdated(address indexed token, uint256 coolDownSeconds);
    event UserMaxTransferUpdated(address indexed token, address indexed user, uint256 amount);
    event UserCooldownUpdated(address indexed token, address indexed user, uint256 coolDownSeconds);
    event ExemptionUpdated(address indexed token, address indexed account, bool status);
    event ContractAuthorized(address indexed contractAddress);
    event LimitManagerAdded(address indexed account);
    event LimitManagerRemoved(address indexed account);

    error ZeroAddress();
    error AmountTooSmall();
    error CooldownNotElapsed(uint256 nextValidTime);
    error ExceedsMaxAmount(uint256 amount, uint256 max);
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(LIMIT_MANAGER_ROLE, msg.sender);
    }
    
    function setDefaultMaxTransferAmount(address token, uint256 amount) external onlyRole(ADMIN_ROLE) {
        if (amount == 0) revert AmountTooSmall();
        defaultMaxTransferAmount[token] = amount;
        emit DefaultMaxTransferUpdated(token, amount);
    }
    
    function setDefaultCooldown(address token, uint256 coolDownSeconds) external onlyRole(ADMIN_ROLE) {
        defaultTransferCooldown[token] = coolDownSeconds;
        emit DefaultCooldownUpdated(token, coolDownSeconds);
    }
    
    function setUserMaxTransferAmount(address token, address user, uint256 amount) external onlyRole(LIMIT_MANAGER_ROLE) {
        if (amount == 0) revert AmountTooSmall();
        userMaxTransferAmount[token][user] = amount;
        hasCustomLimits[token][user] = true;
        emit UserMaxTransferUpdated(token, user, amount);
    }
    
    function setUserCooldown(address token, address user, uint256 coolDownSeconds) external onlyRole(LIMIT_MANAGER_ROLE) {
        userTransferCooldown[token][user] = coolDownSeconds;
        hasCustomCooldown[token][user] = true;
        emit UserCooldownUpdated(token, user, coolDownSeconds);
    }
    
    function resetUserToDefault(address token, address user) external onlyRole(LIMIT_MANAGER_ROLE) {
        hasCustomLimits[token][user] = false;
        hasCustomCooldown[token][user] = false;
        delete userMaxTransferAmount[token][user];
        delete userTransferCooldown[token][user];
        
        emit UserMaxTransferUpdated(token, user, defaultMaxTransferAmount[token]);
        emit UserCooldownUpdated(token, user, defaultTransferCooldown[token]);
    }
    
    function setExemption(address token, address account, bool status) external onlyRole(ADMIN_ROLE) {
        exemptFromLimits[token][account] = status;
        emit ExemptionUpdated(token, account, status);
    }
    
    function authorizeContract(address contractAddress) external onlyRole(ADMIN_ROLE) {
        if (contractAddress == address(0)) revert ZeroAddress();
        _grantRole(CONTRACT_ROLE, contractAddress);
        emit ContractAuthorized(contractAddress);
    }
    
    function addLimitManager(address account) external onlyRole(ADMIN_ROLE) {
        _grantRole(LIMIT_MANAGER_ROLE, account);
        emit LimitManagerAdded(account);
    }
    
    function removeLimitManager(address account) external onlyRole(ADMIN_ROLE) {
        _revokeRole(LIMIT_MANAGER_ROLE, account);
        emit LimitManagerRemoved(account);
    }
    
    function checkTransferLimit(address token, address sender, uint256 amount) 
        external view returns (bool) 
    {
        // Check if exempt from limits
        if (exemptFromLimits[token][sender]) {
            return true;
        }
        
        // Get the applicable max transfer amount
        uint256 maxAmount = hasCustomLimits[token][sender] 
            ? userMaxTransferAmount[token][sender] 
            : defaultMaxTransferAmount[token];
            
        // If no limit set, allow the transfer
        if (maxAmount == 0) {
            return true;
        }
        
        return amount <= maxAmount;
    }
    
    function enforceCooldown(address token, address sender) external returns (bool) {
        // Skip if exempt
        if (exemptFromLimits[token][sender]) {
            return true;
        }
        
        // Get the applicable cooldown period
        uint256 cooldownPeriod = hasCustomCooldown[token][sender]
            ? userTransferCooldown[token][sender]
            : defaultTransferCooldown[token];
            
        // Skip if no cooldown
        if (cooldownPeriod == 0) {
            return true;
        }
        
        uint256 nextValidTime = lastTransferTime[token][sender] + cooldownPeriod;
        if (block.timestamp < nextValidTime) {
            revert CooldownNotElapsed(nextValidTime);
        }
        
        // Update last transfer time
        lastTransferTime[token][sender] = block.timestamp;
        return true;
    }
    
    function batchSetUserLimits(
        address token, 
        address[] calldata users, 
        uint256[] calldata amounts,
        uint256[] calldata cooldowns
    ) external onlyRole(LIMIT_MANAGER_ROLE) {
        require(users.length == amounts.length && users.length == cooldowns.length, "Array length mismatch");
        
        for (uint256 i = 0; i < users.length;) {
            // Set max transfer amount if not zero
            if (amounts[i] > 0) {
                userMaxTransferAmount[token][users[i]] = amounts[i];
                hasCustomLimits[token][users[i]] = true;
                emit UserMaxTransferUpdated(token, users[i], amounts[i]);
            }
            
            // Set cooldown period
            userTransferCooldown[token][users[i]] = cooldowns[i];
            hasCustomCooldown[token][users[i]] = true;
            emit UserCooldownUpdated(token, users[i], cooldowns[i]);
            
            unchecked { ++i; }
        }
    }
    
    function batchSetExemptions(
        address token,
        address[] calldata accounts,
        bool status
    ) external onlyRole(ADMIN_ROLE) {
        for (uint256 i = 0; i < accounts.length;) {
            exemptFromLimits[token][accounts[i]] = status;
            emit ExemptionUpdated(token, accounts[i], status);
            unchecked { ++i; }
        }
    }
    
    function getEffectiveMaxTransferAmount(address token, address user) external view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return type(uint256).max;
        }
        
        return hasCustomLimits[token][user] 
            ? userMaxTransferAmount[token][user] 
            : defaultMaxTransferAmount[token];
    }
    
    function getEffectiveCooldownPeriod(address token, address user) external view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0;
        }
        
        return hasCustomCooldown[token][user]
            ? userTransferCooldown[token][user]
            : defaultTransferCooldown[token];
    }
    
    function getNextValidTransferTime(address token, address user) external view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0; // Can transfer anytime
        }
        
        uint256 cooldownPeriod = hasCustomCooldown[token][user]
            ? userTransferCooldown[token][user]
            : defaultTransferCooldown[token];
            
        if (cooldownPeriod == 0) {
            return 0; // Can transfer anytime
        }
        
        uint256 nextTime = lastTransferTime[token][user] + cooldownPeriod;
        return nextTime > block.timestamp ? nextTime : 0;
    }
}