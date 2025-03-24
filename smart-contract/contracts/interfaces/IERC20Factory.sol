// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IERC20Factory {
    function isTokenCreatedByFactory(address token) external view returns (bool);
    function tokenRatios(address token) external view returns (uint256);
}