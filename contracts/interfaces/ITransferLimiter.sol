// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ITransferLimiter
 * @dev Interface for the TransferLimiter contract
 */
interface ITransferLimiter {
    function checkTransferLimit(address token, address sender, uint256 amount) external view returns (bool);
    function enforceCooldown(address token, address sender) external returns (bool);
    function recordTransfer(address token, address sender, uint256 amount) external returns (bool);
    function authorizeContract(address contractAddress) external;
    function setExemption(address token, address account, bool status) external;
    function setDefaultMaxTransferAmount(address token, uint256 amount) external;
    function setDefaultCooldown(address token, uint256 coolDownSeconds) external;
    function setDefaultPeriodLimit(address token, uint256 amount, uint256 periodSeconds) external;
}
