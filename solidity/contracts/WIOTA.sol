// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "../openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "../openzeppelin-contracts/contracts/access/Ownable.sol";

// Wrapped IOTA token contract 

contract WIOTA is ERC20, ERC20Burnable, Ownable {

    constructor()
        ERC20("Wrapped IOTA", "WIOTA")
        Ownable(msg.sender)
    {}

    function mint(address to, uint256 amount) external onlyOwner {
        _mint(to, amount);
    }
}