// SPDX-License-Identifier: MIT
// Version: 2.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "../interfaces/ITransferLimiter.sol";

/**
 * @title TransferLimiter
 * @dev Controls transfer limits and cooldowns for tokens
 * @notice Optimized for gas usage and integration with StableCoin
 */
contract TransferLimiter is ITransferLimiter, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant CONTRACT_ROLE = keccak256("CONTRACT_ROLE");
    bytes32 public constant LIMIT_MANAGER_ROLE = keccak256("LIMIT_MANAGER_ROLE");
    
    struct LimitConfig {
        uint256 maxTransferAmount;   // Maximum amount per single transfer
        uint256 cooldownPeriod;      // Time between transfers (in seconds)
        uint256 periodLimit;         // Maximum amount per period
        uint256 periodDuration;      // Length of period (in seconds)
    }
    
    struct UserLimitConfig {
        uint256 maxTransferAmount;   // User's max transfer amount (if custom)
        uint256 cooldownPeriod;      // User's cooldown period (if custom)
        uint256 periodLimit;         // User's period limit (if custom)
        uint256 periodDuration;      // User's period duration (if custom)
        bool hasCustomLimits;        // Whether user has custom limits
        bool hasCustomCooldown;      // Whether user has custom cooldown
        bool hasCustomPeriodLimit;   // Whether user has custom period limit
    }
    
    struct TransferState {
        uint256 lastTransferTime;    // Last time user made a transfer
        uint256 periodTotal;         // Total transferred in current period
        uint256 periodResetTime;     // When current period ends
        bool exempt;                 // Whether exempt from limits
    }
    
    // Token-wide default settings
    mapping(address => LimitConfig) public defaultLimits;
    
    // Per-user settings and state
    mapping(address => mapping(address => UserLimitConfig)) public userLimits;
    mapping(address => mapping(address => TransferState)) public transferState;
    
    // Constants
    uint256 public constant MAX_BATCH_SIZE = 100;
    
    // Events (condensed for optimization)
    event LimitUpdated(address indexed token, address indexed user, uint256 maxAmount, uint256 cooldown, uint256 periodLimit, uint256 periodDuration, bool isDefault);
    event ExemptionUpdated(address indexed token, address indexed account, bool status);
    event ContractAuthorized(address indexed contractAddress);
    event LimitManagerUpdated(address indexed account, bool added);
    event TransferRecorded(address indexed token, address indexed user, uint256 amount, uint256 periodTotal);
    event PeriodReset(address indexed token, address indexed user, uint256 newResetTime);

    // Custom errors
    error ZeroAddress();
    error AmountTooSmall();
    error CooldownNotElapsed(uint256 nextValidTime);
    error ExceedsSingleTransferLimit(uint256 amount, uint256 max);
    error ExceedsPeriodLimit(uint256 amount, uint256 currentTotal, uint256 periodLimit);
    error ArrayLengthMismatch();
    error BatchTooLarge(uint256 size, uint256 maxSize);
    
    /**
     * @dev Constructor to setup initial roles
     */
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(LIMIT_MANAGER_ROLE, msg.sender);
    }
    
    /**
     * @dev Set default maximum transfer amount for a token
     * @param token Token address
     * @param amount Maximum amount per transfer
     */
    function setDefaultMaxTransferAmount(address token, uint256 amount) external onlyRole(ADMIN_ROLE) {
        if (amount == 0) revert AmountTooSmall();
        defaultLimits[token].maxTransferAmount = amount;
        emit LimitUpdated(token, address(0), amount, defaultLimits[token].cooldownPeriod, defaultLimits[token].periodLimit, defaultLimits[token].periodDuration, true);
    }
    
    /**
     * @dev Set default cooldown period for a token
     * @param token Token address
     * @param coolDownSeconds Cooldown in seconds
     */
    function setDefaultCooldown(address token, uint256 coolDownSeconds) external onlyRole(ADMIN_ROLE) {
        defaultLimits[token].cooldownPeriod = coolDownSeconds;
        emit LimitUpdated(token, address(0), defaultLimits[token].maxTransferAmount, coolDownSeconds, defaultLimits[token].periodLimit, defaultLimits[token].periodDuration, true);
    }
    
    /**
     * @dev Set default period limits for a token
     * @param token Token address
     * @param amount Maximum amount per period
     * @param periodSeconds Period duration in seconds
     */
    function setDefaultPeriodLimit(address token, uint256 amount, uint256 periodSeconds) external onlyRole(ADMIN_ROLE) {
        defaultLimits[token].periodLimit = amount;
        defaultLimits[token].periodDuration = periodSeconds;
        emit LimitUpdated(token, address(0), defaultLimits[token].maxTransferAmount, defaultLimits[token].cooldownPeriod, amount, periodSeconds, true);
    }
    
    /**
     * @dev Set all default limits for a token in one call
     * @param token Token address
     * @param config Limit configuration
     */
    function setAllDefaultLimits(address token, LimitConfig calldata config) external onlyRole(ADMIN_ROLE) {
        if (config.maxTransferAmount == 0) revert AmountTooSmall();
        defaultLimits[token] = config;
        emit LimitUpdated(token, address(0), config.maxTransferAmount, config.cooldownPeriod, config.periodLimit, config.periodDuration, true);
    }
    
    /**
     * @dev Set custom maximum transfer amount for a user
     * @param token Token address
     * @param user User address
     * @param amount Maximum amount per transfer
     */
    function setUserMaxTransferAmount(address token, address user, uint256 amount) external onlyRole(LIMIT_MANAGER_ROLE) {
        if (amount == 0) revert AmountTooSmall();
        userLimits[token][user].maxTransferAmount = amount;
        userLimits[token][user].hasCustomLimits = true;
        emit LimitUpdated(token, user, amount, 
            userLimits[token][user].hasCustomCooldown ? userLimits[token][user].cooldownPeriod : defaultLimits[token].cooldownPeriod,
            userLimits[token][user].hasCustomPeriodLimit ? userLimits[token][user].periodLimit : defaultLimits[token].periodLimit,
            userLimits[token][user].hasCustomPeriodLimit ? userLimits[token][user].periodDuration : defaultLimits[token].periodDuration,
            false);
    }
    
    /**
     * @dev Set custom cooldown period for a user
     * @param token Token address
     * @param user User address
     * @param coolDownSeconds Cooldown in seconds
     */
    function setUserCooldown(address token, address user, uint256 coolDownSeconds) external onlyRole(LIMIT_MANAGER_ROLE) {
        userLimits[token][user].cooldownPeriod = coolDownSeconds;
        userLimits[token][user].hasCustomCooldown = true;
        emit LimitUpdated(token, user, 
            userLimits[token][user].hasCustomLimits ? userLimits[token][user].maxTransferAmount : defaultLimits[token].maxTransferAmount,
            coolDownSeconds,
            userLimits[token][user].hasCustomPeriodLimit ? userLimits[token][user].periodLimit : defaultLimits[token].periodLimit,
            userLimits[token][user].hasCustomPeriodLimit ? userLimits[token][user].periodDuration : defaultLimits[token].periodDuration,
            false);
    }
    
    /**
     * @dev Set custom period limits for a user
     * @param token Token address
     * @param user User address
     * @param amount Maximum amount per period
     * @param periodSeconds Period duration in seconds
     */
    function setUserPeriodLimit(address token, address user, uint256 amount, uint256 periodSeconds) external onlyRole(LIMIT_MANAGER_ROLE) {
        userLimits[token][user].periodLimit = amount;
        userLimits[token][user].periodDuration = periodSeconds;
        userLimits[token][user].hasCustomPeriodLimit = true;
        emit LimitUpdated(token, user, 
            userLimits[token][user].hasCustomLimits ? userLimits[token][user].maxTransferAmount : defaultLimits[token].maxTransferAmount,
            userLimits[token][user].hasCustomCooldown ? userLimits[token][user].cooldownPeriod : defaultLimits[token].cooldownPeriod,
            amount, periodSeconds, false);
    }
    
    /**
     * @dev Set all limits for a user in one call
     * @param token Token address
     * @param user User address
     * @param config User limit configuration
     */
    function setAllUserLimits(address token, address user, UserLimitConfig calldata config) external onlyRole(LIMIT_MANAGER_ROLE) {
        if (config.hasCustomLimits && config.maxTransferAmount == 0) revert AmountTooSmall();
        userLimits[token][user] = config;
        emit LimitUpdated(token, user, 
            config.hasCustomLimits ? config.maxTransferAmount : defaultLimits[token].maxTransferAmount,
            config.hasCustomCooldown ? config.cooldownPeriod : defaultLimits[token].cooldownPeriod,
            config.hasCustomPeriodLimit ? config.periodLimit : defaultLimits[token].periodLimit,
            config.hasCustomPeriodLimit ? config.periodDuration : defaultLimits[token].periodDuration,
            false);
    }
    
    /**
     * @dev Reset user to default limits
     * @param token Token address
     * @param user User address
     */
    function resetUserToDefault(address token, address user) external onlyRole(LIMIT_MANAGER_ROLE) {
        delete userLimits[token][user];
        emit LimitUpdated(token, user, 
            defaultLimits[token].maxTransferAmount,
            defaultLimits[token].cooldownPeriod,
            defaultLimits[token].periodLimit,
            defaultLimits[token].periodDuration,
            true);
    }
    
    /**
     * @dev Reset user's period counter
     * @param token Token address
     * @param user User address
     */
    function resetUserPeriod(address token, address user) external onlyRole(LIMIT_MANAGER_ROLE) {
        transferState[token][user].periodTotal = 0;
        uint256 periodDuration = _getEffectivePeriodDuration(token, user);
        transferState[token][user].periodResetTime = block.timestamp + periodDuration;
        emit PeriodReset(token, user, transferState[token][user].periodResetTime);
    }
    
    /**
     * @dev Set exemption status for an account
     * @param token Token address
     * @param account Account address
     * @param status Exemption status
     */
    function setExemption(address token, address account, bool status) external onlyRole(ADMIN_ROLE) {
        transferState[token][account].exempt = status;
        emit ExemptionUpdated(token, account, status);
    }
    
    /**
     * @dev Authorize a contract to call limiter functions
     * @param contractAddress Contract to authorize
     */
    function authorizeContract(address contractAddress) external onlyRole(ADMIN_ROLE) {
        if (contractAddress == address(0)) revert ZeroAddress();
        _grantRole(CONTRACT_ROLE, contractAddress);
        emit ContractAuthorized(contractAddress);
    }
    
    /**
     * @dev Add a limit manager
     * @param account Account to add as limit manager
     */
    function addLimitManager(address account) external onlyRole(ADMIN_ROLE) {
        _grantRole(LIMIT_MANAGER_ROLE, account);
        emit LimitManagerUpdated(account, true);
    }
    
    /**
     * @dev Remove a limit manager
     * @param account Account to remove as limit manager
     */
    function removeLimitManager(address account) external onlyRole(ADMIN_ROLE) {
        _revokeRole(LIMIT_MANAGER_ROLE, account);
        emit LimitManagerUpdated(account, false);
    }
    
    /**
     * @dev Batch set user limits
     * @param token Token address
     * @param users User addresses
     * @param configs User limit configurations
     */
    function batchSetUserLimits(address token, address[] calldata users, UserLimitConfig[] calldata configs) external onlyRole(LIMIT_MANAGER_ROLE) {
        uint256 length = users.length;
        if (length != configs.length) revert ArrayLengthMismatch();
        if (length > MAX_BATCH_SIZE) revert BatchTooLarge(length, MAX_BATCH_SIZE);
        
        for (uint256 i = 0; i < length;) {
            if (configs[i].hasCustomLimits && configs[i].maxTransferAmount == 0) revert AmountTooSmall();
            userLimits[token][users[i]] = configs[i];
            emit LimitUpdated(token, users[i], 
                configs[i].hasCustomLimits ? configs[i].maxTransferAmount : defaultLimits[token].maxTransferAmount,
                configs[i].hasCustomCooldown ? configs[i].cooldownPeriod : defaultLimits[token].cooldownPeriod,
                configs[i].hasCustomPeriodLimit ? configs[i].periodLimit : defaultLimits[token].periodLimit,
                configs[i].hasCustomPeriodLimit ? configs[i].periodDuration : defaultLimits[token].periodDuration,
                false);
            unchecked { ++i; }
        }
    }
    
    /**
     * @dev Batch set exemptions
     * @param token Token address
     * @param accounts Account addresses
     * @param status Exemption status
     */
    function batchSetExemptions(address token, address[] calldata accounts, bool status) external onlyRole(ADMIN_ROLE) {
        uint256 length = accounts.length;
        if (length > MAX_BATCH_SIZE) revert BatchTooLarge(length, MAX_BATCH_SIZE);
        
        for (uint256 i = 0; i < length;) {
            transferState[token][accounts[i]].exempt = status;
            emit ExemptionUpdated(token, accounts[i], status);
            unchecked { ++i; }
        }
    }
    
    /**
     * @dev Check if a transfer is within limits
     * @param token Token address
     * @param sender Sender address
     * @param amount Amount to transfer
     * @return bool Whether the transfer is allowed
     */
    function checkTransferLimit(address token, address sender, uint256 amount) external view returns (bool) {
        // Check if exempt from limits
        if (transferState[token][sender].exempt) {
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
    
    /**
     * @dev Enforce cooldown between transfers
     * @param token Token address
     * @param sender Sender address
     * @return bool Success indicator
     */
    function enforceCooldown(address token, address sender) external onlyRole(CONTRACT_ROLE) returns (bool) {
        // Skip if exempt
        if (transferState[token][sender].exempt) {
            return true;
        }
        
        // Get the applicable cooldown period
        uint256 cooldownPeriod = _getEffectiveCooldownPeriod(token, sender);
            
        // Skip if no cooldown
        if (cooldownPeriod == 0) {
            return true;
        }
        
        uint256 nextValidTime = transferState[token][sender].lastTransferTime + cooldownPeriod;
        if (block.timestamp < nextValidTime) {
            revert CooldownNotElapsed(nextValidTime);
        }
        
        // Update last transfer time
        transferState[token][sender].lastTransferTime = block.timestamp;
        return true;
    }
    
    /**
     * @dev Record a transfer for limit tracking
     * @param token Token address
     * @param sender Sender address
     * @param amount Amount transferred
     * @return bool Success indicator
     */
    function recordTransfer(address token, address sender, uint256 amount) external onlyRole(CONTRACT_ROLE) returns (bool) {
        // Skip if exempt
        if (transferState[token][sender].exempt) {
            return true;
        }
        
        // Check if period has reset
        _checkAndResetPeriod(token, sender);
        
        // Check period limit
        uint256 periodLimit = _getEffectivePeriodLimit(token, sender);
        if (periodLimit > 0) {
            uint256 newTotal = transferState[token][sender].periodTotal + amount;
            if (newTotal > periodLimit) {
                revert ExceedsPeriodLimit(amount, transferState[token][sender].periodTotal, periodLimit);
            }
            
            // Update total transferred
            transferState[token][sender].periodTotal = newTotal;
            emit TransferRecorded(token, sender, amount, newTotal);
        }
        
        return true;
    }
    
    // View functions to get effective limits
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
    
    /**
     * @dev Get next valid transfer time for a user
     * @param token Token address
     * @param user User address
     * @return uint256 Next valid transfer time (0 if can transfer now)
     */
    function getNextValidTransferTime(address token, address user) external view returns (uint256) {
        if (transferState[token][user].exempt) {
            return 0; // Can transfer anytime
        }
        
        uint256 cooldownPeriod = _getEffectiveCooldownPeriod(token, user);
        if (cooldownPeriod == 0) {
            return 0; // Can transfer anytime
        }
        
        uint256 nextTime = transferState[token][user].lastTransferTime + cooldownPeriod;
        return nextTime > block.timestamp ? nextTime : 0;
    }
    
    /**
     * @dev Get remaining period allowance for a user
     * @param token Token address
     * @param user User address
     * @return uint256 Remaining allowance
     * @return uint256 Period reset time
     */
    function getRemainingPeriodAllowance(address token, address user) external view returns (uint256, uint256) {
        if (transferState[token][user].exempt) {
            return (type(uint256).max, 0); // No limit, no reset time
        }
        
        // Check if period has reset (view version)
        uint256 resetTime = transferState[token][user].periodResetTime;
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
        uint256 remaining = periodLimit > transferState[token][user].periodTotal 
            ? periodLimit - transferState[token][user].periodTotal 
            : 0;
            
        return (remaining, resetTime);
    }
    
    // Internal helper functions
    function _getEffectiveMaxTransferAmount(address token, address user) internal view returns (uint256) {
        if (transferState[token][user].exempt) {
            return type(uint256).max;
        }
        
        return userLimits[token][user].hasCustomLimits 
            ? userLimits[token][user].maxTransferAmount 
            : defaultLimits[token].maxTransferAmount;
    }
    
    function _getEffectiveCooldownPeriod(address token, address user) internal view returns (uint256) {
        if (transferState[token][user].exempt) {
            return 0;
        }
        
        return userLimits[token][user].hasCustomCooldown
            ? userLimits[token][user].cooldownPeriod
            : defaultLimits[token].cooldownPeriod;
    }
    
    function _getEffectivePeriodLimit(address token, address user) internal view returns (uint256) {
        if (transferState[token][user].exempt) {
            return 0; // No limit
        }
        
        return userLimits[token][user].hasCustomPeriodLimit
            ? userLimits[token][user].periodLimit
            : defaultLimits[token].periodLimit;
    }
    
    function _getEffectivePeriodDuration(address token, address user) internal view returns (uint256) {
        if (transferState[token][user].exempt) {
            return 0; // No period
        }
        
        return userLimits[token][user].hasCustomPeriodLimit
            ? userLimits[token][user].periodDuration
            : defaultLimits[token].periodDuration;
    }
    
    function _checkPeriodLimitView(address token, address user, uint256 amount) internal view {
        uint256 periodLimit = _getEffectivePeriodLimit(token, user);
        if (periodLimit == 0) {
            return; // No period limit
        }
        
        // Check if period has reset (view version)
        uint256 resetTime = transferState[token][user].periodResetTime;
        if (resetTime > 0 && block.timestamp >= resetTime) {
            // Period has reset
            if (amount > periodLimit) {
                revert ExceedsPeriodLimit(amount, 0, periodLimit);
            }
        } else {
            // Period is active
            uint256 newTotal = transferState[token][user].periodTotal + amount;
            if (newTotal > periodLimit) {
                revert ExceedsPeriodLimit(amount, transferState[token][user].periodTotal, periodLimit);
            }
        }
    }
    
    function _checkAndResetPeriod(address token, address user) internal {
        uint256 resetTime = transferState[token][user].periodResetTime;
        uint256 periodDuration = _getEffectivePeriodDuration(token, user);
        
        // If no reset time has been set yet
        if (resetTime == 0 && periodDuration > 0) {
            // Set initial reset time
            transferState[token][user].periodResetTime = block.timestamp + periodDuration;
            return;
        }
        
        // If period is over, reset the counter
        if (resetTime > 0 && block.timestamp >= resetTime) {
            transferState[token][user].periodTotal = 0;
            
            // Set next reset time if we have a period duration
            if (periodDuration > 0) {
                // Calculate next reset time based on current time
                transferState[token][user].periodResetTime = block.timestamp + periodDuration;
                emit PeriodReset(token, user, transferState[token][user].periodResetTime);
            } else {
                // No period duration means no period limit
                transferState[token][user].periodResetTime = 0;
            }
        }
    }
}