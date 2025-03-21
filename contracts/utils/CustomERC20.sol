// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract CustomERC20 is ERC20, AccessControl, Pausable, ReentrancyGuard {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    address public stableCoin;
    address public swapper;
    uint256 public stableCoinRatio; // 1 token = x stablecoins
    
    // Security settings
    mapping(address => bool) public blacklisted;
    
    // Events
    event BlacklistUpdated(address indexed account, bool blacklisted);
    
    constructor(
        string memory name,
        string memory symbol,
        uint256 initialSupply,
        address owner,
        address _stableCoin,
        address _swapper,
        uint256 _stableCoinRatio
    ) ERC20(name, symbol) {
        _mint(owner, initialSupply * 10**decimals());
        
        _grantRole(DEFAULT_ADMIN_ROLE, owner);
        _grantRole(MINTER_ROLE, owner);
        _grantRole(BURNER_ROLE, owner);
        _grantRole(PAUSER_ROLE, owner);
        
        stableCoin = _stableCoin;
        swapper = _swapper;
        stableCoinRatio = _stableCoinRatio;
    }
    
    function grantMinterRole(address account) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _grantRole(MINTER_ROLE, account);
        _grantRole(BURNER_ROLE, account);
    }
    
    function setBlacklisted(address account, bool isBlacklisted) external onlyRole(DEFAULT_ADMIN_ROLE) {
        blacklisted[account] = isBlacklisted;
        emit BlacklistUpdated(account, isBlacklisted);
    }
    
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }
    
    function mint(address to, uint256 amount) public onlyRole(MINTER_ROLE) {
        require(!blacklisted[to], "Recipient is blacklisted");
        _mint(to, amount);
    }
    
    function burn(uint256 amount) public {
        require(!blacklisted[msg.sender], "Account is blacklisted");
        _burn(msg.sender, amount);
    }
    
    function burnFrom(address account, uint256 amount) public {
        require(!blacklisted[msg.sender], "Sender is blacklisted");
        require(!blacklisted[account], "Account is blacklisted");
        
        uint256 currentAllowance = allowance(account, msg.sender);
        require(currentAllowance >= amount, "ERC20: burn amount exceeds allowance");
        
        unchecked {
            _approve(account, msg.sender, currentAllowance - amount);
        }
        _burn(account, amount);
    }
    
    // Override transfer and transferFrom without maxTransferAmount and transferCooldown checks
    function transfer(address to, uint256 amount) public override whenNotPaused returns (bool) {
        require(!blacklisted[msg.sender], "Sender is blacklisted");
        require(!blacklisted[to], "Recipient is blacklisted");
        
        return super.transfer(to, amount);
    }
    
    function transferFrom(address from, address to, uint256 amount) public override whenNotPaused returns (bool) {
        require(!blacklisted[from], "Sender is blacklisted");
        require(!blacklisted[to], "Recipient is blacklisted");
        require(!blacklisted[msg.sender], "Operator is blacklisted");
        
        return super.transferFrom(from, to, amount);
    }
    
    // Emergency recovery function
    function recoverERC20(address tokenAddress, uint256 tokenAmount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(tokenAddress != address(this), "Cannot recover the token itself");
        IERC20(tokenAddress).transfer(msg.sender, tokenAmount);
    }
}