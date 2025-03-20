// SPDX-License-Identifier: MIT
// Version: 1.2.0
// Last updated: 2025-03-20 06:04:28
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title TransferLimiter
 * @dev Controls transfer limits and cooldowns for tokens
 * @notice Supports period-based transfer limits (daily/hourly limits)
 */
contract TransferLimiter is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant CONTRACT_ROLE = keccak256("CONTRACT_ROLE");
    bytes32 public constant LIMIT_MANAGER_ROLE = keccak256("LIMIT_MANAGER_ROLE");
    
    // Token-wide settings
    mapping(address => uint256) public defaultMaxTransferAmount;
    mapping(address => uint256) public defaultTransferCooldown;
    mapping(address => uint256) public defaultPeriodLimit;
    mapping(address => uint256) public defaultPeriodDuration;
    
    // Per-user settings
    mapping(address => mapping(address => uint256)) public userMaxTransferAmount;
    mapping(address => mapping(address => uint256)) public userTransferCooldown;
    mapping(address => mapping(address => uint256)) public userPeriodLimit;
    mapping(address => mapping(address => uint256)) public userPeriodDuration;
    
    // Has custom settings
    mapping(address => mapping(address => bool)) public hasCustomLimits;
    mapping(address => mapping(address => bool)) public hasCustomCooldown;
    mapping(address => mapping(address => bool)) public hasCustomPeriodLimit;
    
    // Transfer tracking
    mapping(address => mapping(address => uint256)) public lastTransferTime;
    mapping(address => mapping(address => uint256)) public periodTotalTransferred;
    mapping(address => mapping(address => uint256)) public periodResetTime;
    
    // Exemptions
    mapping(address => mapping(address => bool)) public exemptFromLimits;
    
    // Events
    event DefaultMaxTransferUpdated(address indexed token, uint256 amount);
    event DefaultCooldownUpdated(address indexed token, uint256 coolDownSeconds);
    event DefaultPeriodLimitUpdated(address indexed token, uint256 amount, uint256 periodSeconds);
    event UserMaxTransferUpdated(address indexed token, address indexed user, uint256 amount);
    event UserCooldownUpdated(address indexed token, address indexed user, uint256 coolDownSeconds);
    event UserPeriodLimitUpdated(address indexed token, address indexed user, uint256 amount, uint256 periodSeconds);
    event ExemptionUpdated(address indexed token, address indexed account, bool status);
    event ContractAuthorized(address indexed contractAddress);
    event LimitManagerAdded(address indexed account);
    event LimitManagerRemoved(address indexed account);
    event TransferRecorded(address indexed token, address indexed user, uint256 amount, uint256 periodTotal);
    event PeriodReset(address indexed token, address indexed user, uint256 newResetTime);

    // Custom errors
    error ZeroAddress();
    error AmountTooSmall();
    error CooldownNotElapsed(uint256 nextValidTime);
    error ExceedsSingleTransferLimit(uint256 amount, uint256 max);
    error ExceedsPeriodLimit(uint256 amount, uint256 currentTotal, uint256 periodLimit);
    error ArrayLengthMismatch();
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(LIMIT_MANAGER_ROLE, msg.sender);
    }
    
    // ========== DEFAULT SETTINGS FUNCTIONS ==========
    
    function setDefaultMaxTransferAmount(address token, uint256 amount) external onlyRole(ADMIN_ROLE) {
        if (amount == 0) revert AmountTooSmall();
        defaultMaxTransferAmount[token] = amount;
        emit DefaultMaxTransferUpdated(token, amount);
    }
    
    function setDefaultCooldown(address token, uint256 coolDownSeconds) external onlyRole(ADMIN_ROLE) {
        defaultTransferCooldown[token] = coolDownSeconds;
        emit DefaultCooldownUpdated(token, coolDownSeconds);
    }
    
    function setDefaultPeriodLimit(address token, uint256 amount, uint256 periodSeconds) external onlyRole(ADMIN_ROLE) {
        defaultPeriodLimit[token] = amount;
        defaultPeriodDuration[token] = periodSeconds;
        emit DefaultPeriodLimitUpdated(token, amount, periodSeconds);
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
    
    function setUserPeriodLimit(address token, address user, uint256 amount, uint256 periodSeconds) external onlyRole(LIMIT_MANAGER_ROLE) {
        userPeriodLimit[token][user] = amount;
        userPeriodDuration[token][user] = periodSeconds;
        hasCustomPeriodLimit[token][user] = true;
        emit UserPeriodLimitUpdated(token, user, amount, periodSeconds);
    }
    
    function resetUserToDefault(address token, address user) external onlyRole(LIMIT_MANAGER_ROLE) {
        hasCustomLimits[token][user] = false;
        hasCustomCooldown[token][user] = false;
        hasCustomPeriodLimit[token][user] = false;
        
        delete userMaxTransferAmount[token][user];
        delete userTransferCooldown[token][user];
        delete userPeriodLimit[token][user];
        delete userPeriodDuration[token][user];
        
        emit UserMaxTransferUpdated(token, user, defaultMaxTransferAmount[token]);
        emit UserCooldownUpdated(token, user, defaultTransferCooldown[token]);
        emit UserPeriodLimitUpdated(token, user, defaultPeriodLimit[token], defaultPeriodDuration[token]);
    }
    
    function resetUserPeriod(address token, address user) external onlyRole(LIMIT_MANAGER_ROLE) {
        periodTotalTransferred[token][user] = 0;
        periodResetTime[token][user] = block.timestamp + _getEffectivePeriodDuration(token, user);
        emit PeriodReset(token, user, periodResetTime[token][user]);
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
    
    function batchSetUserLimits(
        address token, 
        address[] calldata users, 
        uint256[] calldata amounts,
        uint256[] calldata cooldowns,
        uint256[] calldata periodLimits,
        uint256[] calldata periodDurations
    ) external onlyRole(LIMIT_MANAGER_ROLE) {
        if (
            users.length != amounts.length || 
            users.length != cooldowns.length ||
            users.length != periodLimits.length ||
            users.length != periodDurations.length
        ) revert ArrayLengthMismatch();
        
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
            
            // Set period limits
            if (periodLimits[i] > 0) {
                userPeriodLimit[token][users[i]] = periodLimits[i];
                userPeriodDuration[token][users[i]] = periodDurations[i];
                hasCustomPeriodLimit[token][users[i]] = true;
                emit UserPeriodLimitUpdated(token, users[i], periodLimits[i], periodDurations[i]);
            }
            
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
    
    function checkTransferLimit(address token, address sender, uint256 amount) 
        external view returns (bool) 
    {
        // Check if exempt from limits
        if (exemptFromLimits[token][sender]) {
            return true;
        }
        
        // Check single transfer limit
        uint256 maxAmount = _getEffectiveMaxTransferAmount(token, sender);
        if (maxAmount > 0 && amount > maxAmount) {
            return false;
        }
        
        // Check period limit
        _checkPeriodLimitView(token, sender, amount);
        
        return true;
    }
    
    function enforceCooldown(address token, address sender) external returns (bool) {
        // Skip if exempt
        if (exemptFromLimits[token][sender]) {
            return true;
        }
        
        // Get the applicable cooldown period
        uint256 cooldownPeriod = _getEffectiveCooldownPeriod(token, sender);
            
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
    
    function recordTransfer(address token, address sender, uint256 amount) external onlyRole(CONTRACT_ROLE) returns (bool) {
        // Skip if exempt
        if (exemptFromLimits[token][sender]) {
            return true;
        }
        
        // Check if period has reset
        _checkAndResetPeriod(token, sender);
        
        // Check period limit
        uint256 periodLimit = _getEffectivePeriodLimit(token, sender);
        if (periodLimit > 0) {
            uint256 newTotal = periodTotalTransferred[token][sender] + amount;
            if (newTotal > periodLimit) {
                revert ExceedsPeriodLimit(amount, periodTotalTransferred[token][sender], periodLimit);
            }
            
            // Update total transferred
            periodTotalTransferred[token][sender] = newTotal;
            emit TransferRecorded(token, sender, amount, newTotal);
        }
        
        return true;
    }
    
    function getEffectiveMaxTransferAmount(address token, address user) external view returns (uint256) {
        return _getEffectiveMaxTransferAmount(token, user);
    }
    
    function getEffectiveCooldownPeriod(address token, address user) external view returns (uint256) {
        return _getEffectiveCooldownPeriod(token, user);
    }
    
    function getEffectivePeriodLimit(address token, address user) external view returns (uint256) {
        return _getEffectivePeriodLimit(token, user);
    }
    
    function getEffectivePeriodDuration(address token, address user) external view returns (uint256) {
        return _getEffectivePeriodDuration(token, user);
    }
    
    function getNextValidTransferTime(address token, address user) external view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0; // Can transfer anytime
        }
        
        uint256 cooldownPeriod = _getEffectiveCooldownPeriod(token, user);
        if (cooldownPeriod == 0) {
            return 0; // Can transfer anytime
        }
        
        uint256 nextTime = lastTransferTime[token][user] + cooldownPeriod;
        return nextTime > block.timestamp ? nextTime : 0;
    }
    
    function getRemainingPeriodAllowance(address token, address user) external view returns (uint256, uint256) {
        if (exemptFromLimits[token][user]) {
            return (type(uint256).max, 0); // No limit, no reset time
        }
        
        // Check if period has reset (view version)
        uint256 resetTime = periodResetTime[token][user];
        uint256 periodDuration = _getEffectivePeriodDuration(token, user);
        
        // If no reset time has been set yet, or if period is over
        if (resetTime == 0 || (resetTime > 0 && block.timestamp >= resetTime)) {
            // Period has reset or not started
            uint256 _periodLimit = _getEffectivePeriodLimit(token, user);
            uint256 newResetTime = periodDuration > 0 ? block.timestamp + periodDuration : 0;
            return (_periodLimit, newResetTime);
        }
        
        // Period is active
        uint256 periodLimit = _getEffectivePeriodLimit(token, user);
        uint256 remaining = periodLimit > periodTotalTransferred[token][user] 
            ? periodLimit - periodTotalTransferred[token][user] 
            : 0;
            
        return (remaining, resetTime);
    }
    
    function _getEffectiveMaxTransferAmount(address token, address user) internal view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return type(uint256).max;
        }
        
        return hasCustomLimits[token][user] 
            ? userMaxTransferAmount[token][user] 
            : defaultMaxTransferAmount[token];
    }
    
    function _getEffectiveCooldownPeriod(address token, address user) internal view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0;
        }
        
        return hasCustomCooldown[token][user]
            ? userTransferCooldown[token][user]
            : defaultTransferCooldown[token];
    }
    
    function _getEffectivePeriodLimit(address token, address user) internal view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0; // No limit
        }
        
        return hasCustomPeriodLimit[token][user]
            ? userPeriodLimit[token][user]
            : defaultPeriodLimit[token];
    }
    
    function _getEffectivePeriodDuration(address token, address user) internal view returns (uint256) {
        if (exemptFromLimits[token][user]) {
            return 0; // No period
        }
        
        return hasCustomPeriodLimit[token][user]
            ? userPeriodDuration[token][user]
            : defaultPeriodDuration[token];
    }
    
    function _checkPeriodLimitView(address token, address user, uint256 amount) internal view {
        uint256 periodLimit = _getEffectivePeriodLimit(token, user);
        if (periodLimit == 0) {
            return; // No period limit
        }
        
        // Check if period has reset (view version)
        uint256 resetTime = periodResetTime[token][user];
        if (resetTime > 0 && block.timestamp >= resetTime) {
            // Period has reset
            if (amount > periodLimit) {
                revert ExceedsPeriodLimit(amount, 0, periodLimit);
            }
        } else {
            // Period is active
            uint256 newTotal = periodTotalTransferred[token][user] + amount;
            if (newTotal > periodLimit) {
                revert ExceedsPeriodLimit(amount, periodTotalTransferred[token][user], periodLimit);
            }
        }
    }
    
    function _checkAndResetPeriod(address token, address user) internal {
        uint256 resetTime = periodResetTime[token][user];
        uint256 periodDuration = _getEffectivePeriodDuration(token, user);
        
        // If no reset time has been set yet
        if (resetTime == 0 && periodDuration > 0) {
            // Set initial reset time
            periodResetTime[token][user] = block.timestamp + periodDuration;
            return;
        }
        
        // If period is over, reset the counter
        if (resetTime > 0 && block.timestamp >= resetTime) {
            periodTotalTransferred[token][user] = 0;
            
            // Set next reset time if we have a period duration
            if (periodDuration > 0) {
                // Calculate next reset time based on current time
                periodResetTime[token][user] = block.timestamp + periodDuration;
                emit PeriodReset(token, user, periodResetTime[token][user]);
            } else {
                // No period duration means no period limit
                periodResetTime[token][user] = 0;
            }
        }
    }
}