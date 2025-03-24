// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

contract StableCoin is ERC20, AccessControl, Pausable, ReentrancyGuard {
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant WHITELIST_MANAGER_ROLE =
        keccak256("WHITELIST_MANAGER_ROLE");

    mapping(address => bool) public whitelisted;
    bool public enforceWhitelistForReceivers;

    event Whitelisted(address indexed account, bool isWhitelisted);
    event WhitelistReceiverPolicyChanged(bool enforceForReceivers);
    event withdrawEvent(uint256 amount, address withdrawer, bytes32 data);

    constructor(
        string memory name,
        string memory symbol,
        uint256 initialSupply
    ) ERC20(name, symbol) {
        _mint(msg.sender, initialSupply * 10 ** decimals());
        enforceWhitelistForReceivers = true;

        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(PAUSER_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(BURNER_ROLE, msg.sender);
        _grantRole(WHITELIST_MANAGER_ROLE, msg.sender);
    }

    function addToWhitelist(
        address account
    ) external onlyRole(WHITELIST_MANAGER_ROLE) {
        whitelisted[account] = true;
        emit Whitelisted(account, true);
    }

    function removeFromWhitelist(
        address account
    ) external onlyRole(WHITELIST_MANAGER_ROLE) {
        whitelisted[account] = false;
        emit Whitelisted(account, false);
    }

    function batchAddToWhitelist(
        address[] calldata accounts
    ) external onlyRole(WHITELIST_MANAGER_ROLE) {
        for (uint256 i = 0; i < accounts.length; i++) {
            whitelisted[accounts[i]] = true;
            emit Whitelisted(accounts[i], true);
        }
    }

    function setWhitelistReceiverPolicy(
        bool enforceForReceivers
    ) external onlyRole(ADMIN_ROLE) {
        enforceWhitelistForReceivers = enforceForReceivers;
        emit WhitelistReceiverPolicyChanged(enforceForReceivers);
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function mint(
        address to,
        uint256 amount
    ) external onlyRole(MINTER_ROLE) nonReentrant {
        _mint(to, amount);
    }

    function burn(uint256 amount) external onlyRole(BURNER_ROLE) nonReentrant {
        _burn(msg.sender, amount);
    }

    function withdraw(
        uint256 amount,
        address withdrawer,
        bytes32 data
    ) external onlyRole(BURNER_ROLE) nonReentrant {
        _burn(withdrawer, amount);
        emit withdrawEvent(amount, withdrawer, data);
    }

    function _update(
        address from,
        address to,
        uint256 amount
    ) internal override whenNotPaused {
        // Skip checks for minting (from is zero address) and burning (to is zero address)
        if (from != address(0) && to != address(0)) {
            // Always require sender to be whitelisted
            require(whitelisted[from], "Sender not whitelisted");

            // Optionally require receiver to be whitelisted based on policy
            if (enforceWhitelistForReceivers) {
                require(whitelisted[to], "Receiver not whitelisted");
            }
        }

        super._update(from, to, amount);
    }
}
