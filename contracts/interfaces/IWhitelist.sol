// SPDX-License-Identifier: MIT 
pragma solidity ^0.8.20;

/**
 * @title IWhitelist
 * @dev Interface for the Whitelist contract
 */
interface IWhitelist {
    function checkWhitelist(address account) external view returns (bool);
    function isWhitelisted(address account) external view returns (bool);
    function authorizeContract(address contractAddress) external;
    function setWhitelisted(address account, bool status) external;
    function batchSetWhitelisted(address[] calldata accounts, bool status) external;
    function toggleWhitelisting(bool enabled) external;
}
