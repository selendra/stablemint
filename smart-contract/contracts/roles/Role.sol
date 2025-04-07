// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

contract Common {
    // More descriptive error messages
    error ZeroAddressProvided(string reason);
    error InvalidNumber(string reason, uint256 providedValue);

    modifier isNotZeroAddress(address _account) {
        if (_account == address(0))
            revert ZeroAddressProvided("Address cannot be zero");
        _;
    }

    modifier isValNumber(uint256 _amount) {
        if (_amount <= 0)
            revert InvalidNumber("Amount must be greater than zero", _amount);
        _;
    }
}

contract Capper is ReentrancyGuard {
    uint256 private constant INITIAL_CAPACITY = 0;
    uint256 public capacity = INITIAL_CAPACITY;

    event Cap(uint256 indexed newCapacity, address indexed sender);

    error AmountExceedsCapacity(
        uint256 amount,
        uint256 currentCapacity,
        string reason
    );

    error CapIsNotUpdate(
        uint256 amount,
        uint256 currentCapacity,
        string reason
    );

    modifier notMoreThanCapacity(uint256 _amount) {
        if (_amount > capacity)
            revert AmountExceedsCapacity(
                _amount,
                capacity,
                "Requested amount exceeds available capacity"
            );
        _;
    }

    function _cap(uint256 _amount) internal {
        if (capacity == _amount) {
            revert CapIsNotUpdate(
                _amount,
                capacity,
                "Requested amount same as capacity"
            );
        }
        capacity = _amount;
        emit Cap(capacity, msg.sender);
    }
}

contract AdvaRoleController is AccessControl, Capper, Common {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant CAPPER_ROLE = keccak256("CAPPER_ROLE");
    bytes32 public constant RECOVER_ROLE = keccak256("RECOVER_ROLE");
    bytes32 public constant BANNER_ROLE = keccak256("BANNER_ROLE");
    bytes32 public constant PAUASER_ROLE = keccak256("PAUASER_ROLE");

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(CAPPER_ROLE, msg.sender);
        _grantRole(RECOVER_ROLE, msg.sender);
        _grantRole(BANNER_ROLE, msg.sender);
        _grantRole(PAUASER_ROLE, msg.sender);
    }

    function setCap(
        uint256 _amount
    ) public onlyRole(CAPPER_ROLE) isValNumber(_amount) nonReentrant {
        _cap(_amount);
    }
}
