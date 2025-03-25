// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

contract ERC20Token is ERC20, AccessControl, ReentrancyGuard {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    address public immutable factory;
    event RoleAdminChanged(
        bytes32 indexed role,
        address indexed account,
        address indexed caller
    );

    constructor(
        string memory name,
        string memory symbol,
        address tokenOwner
    ) ERC20(name, symbol) {
        require(tokenOwner != address(0), "Token owner cannot be zero address");
        factory = msg.sender;

        // Set up roles
        _grantRole(DEFAULT_ADMIN_ROLE, tokenOwner);
        _grantRole(ADMIN_ROLE, tokenOwner);
        _grantRole(PAUSER_ROLE, tokenOwner);
    }

    function mint(address to, uint256 amount) external nonReentrant {
        require(to != address(0), "Cannot mint to zero address");
        require(amount > 0, "Amount must be greater than zero");
        require(msg.sender == factory, "Only factory can mint");

        _mint(to, amount);
    }

    function burn(uint256 amount) external nonReentrant {
        require(amount > 0, "Amount must be greater than zero");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");

        _burn(msg.sender, amount);
    }

    function burnFrom(address account, uint256 amount) external nonReentrant {
        require(account != address(0), "Cannot burn from zero address");
        require(amount > 0, "Amount must be greater than zero");

        uint256 currentAllowance = allowance(account, msg.sender);
        require(
            currentAllowance >= amount,
            "ERC20: burn amount exceeds allowance"
        );

        unchecked {
            _approve(account, msg.sender, currentAllowance - amount);
        }
        _burn(account, amount);
    }
}
