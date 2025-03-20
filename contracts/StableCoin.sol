// SPDX-License-Identifier: MIT
// Version: 3.3.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./utils/WhiteList.sol";
import "./utils/LimitTransafer.sol";

contract StableCoin is ERC20, AccessControl, Pausable, ReentrancyGuard {
    
    // Roles
    bytes32 private constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 private constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 private constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    // External services
    Whitelist public whitelistManager;
    TransferLimiter public transferLimiter;
    
    // Events
    event WhitelistManagerSet(address indexed managerAddress);
    event TransferLimiterSet(address indexed limiterAddress);
    event TokensMinted(address indexed to, uint256 amount);
    event TokensBurned(address indexed from, uint256 amount);

    // Errors
    error NotWhitelisted(address account);
    error ExceedsMaxAmount(uint256 amount, uint256 max);
    error CooldownNotElapsed(uint256 nextValidTime);
    error CannotRecoverSelf();
    error LimitExceeded();
    error ZeroAddress();
    error NotAuthorized();
    
    constructor(string memory name, string memory symbol, uint256 initialSupply) 
        ERC20(name, symbol) 
    {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(BURNER_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
        
        _mint(msg.sender, initialSupply * 10**decimals());
    }
    
    function setWhitelistManager(address _whitelistManager) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (_whitelistManager == address(0)) revert ZeroAddress();
        whitelistManager = Whitelist(_whitelistManager);
        emit WhitelistManagerSet(_whitelistManager);
    }
    
    function setTransferLimiter(address _transferLimiter) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (_transferLimiter == address(0)) revert ZeroAddress();
        transferLimiter = TransferLimiter(_transferLimiter);
        emit TransferLimiterSet(_transferLimiter);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function mint(address to, uint256 amount) public onlyRole(MINTER_ROLE) {
        _checkWhitelist(to);
        _mint(to, amount);
        emit TokensMinted(to, amount);
    }
    
    function burn(uint256 amount) public virtual onlyRole(BURNER_ROLE) {
        _burn(msg.sender, amount);
        emit TokensBurned(msg.sender, amount);
    }

    function burnFrom(address account, uint256 amount) public virtual onlyRole(BURNER_ROLE) {
        uint256 currentAllowance = allowance(account, msg.sender);
        if (currentAllowance < amount) revert ExceedsMaxAmount(amount, currentAllowance);
        
        unchecked {
            _approve(account, msg.sender, currentAllowance - amount);
        }
        _burn(account, amount);
        emit TokensBurned(account, amount);
    }
    
    function transfer(address to, uint256 amount) public override whenNotPaused returns (bool) {
        _checkWhitelist(msg.sender);
        _checkWhitelist(to);
        _checkTransferLimit(msg.sender, amount);
        _enforceCooldown(msg.sender);
        _recordTransfer(msg.sender, amount);
        
        return super.transfer(to, amount);
    }
    
    function transferFrom(address from, address to, uint256 amount) public override whenNotPaused returns (bool) {
        _checkWhitelist(from);
        _checkWhitelist(to);
        _checkWhitelist(msg.sender);
        _checkTransferLimit(from, amount);
        _enforceCooldown(from);
        _recordTransfer(from, amount);
        
        return super.transferFrom(from, to, amount);
    }
    
    function recoverERC20(address tokenAddress, uint256 amount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (tokenAddress == address(this)) revert CannotRecoverSelf();
        IERC20(tokenAddress).transfer(msg.sender, amount);
    }
    
    function _checkWhitelist(address account) internal view {
        if (address(whitelistManager) == address(0)) {
            return; // Skip check if no whitelist manager set
        }
        
        if (!whitelistManager.checkWhitelist(account)) {
            revert NotWhitelisted(account);
        }
    }
    
    function _checkTransferLimit(address sender, uint256 amount) internal view {
        if (address(transferLimiter) == address(0)) {
            return; // Skip check if no limiter set
        }
        
        if (!transferLimiter.checkTransferLimit(address(this), sender, amount)) {
            revert LimitExceeded();
        }
    }
    
    function _enforceCooldown(address sender) internal {
        if (address(transferLimiter) == address(0)) {
            return; // Skip check if no limiter set
        }
        
        // This will revert with CooldownNotElapsed if cooldown not elapsed
        transferLimiter.enforceCooldown(address(this), sender);
    }
    
    function _recordTransfer(address sender, uint256 amount) internal {
        if (address(transferLimiter) == address(0)) {
            return;
        }
        
        transferLimiter.recordTransfer(address(this), sender, amount);
    }
}