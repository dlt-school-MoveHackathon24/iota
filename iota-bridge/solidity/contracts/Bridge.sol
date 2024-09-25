// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "../openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "../openzeppelin-contracts/contracts/access/Ownable.sol";
import "./WIOTA.sol";

contract Bridge is Ownable {

    event LockETH(address indexed user, string addressIOTA, uint256 amount);
    event MintWIOTA(address indexed user, uint256 amount);
    event BurnWIOTA(address indexed user, string addressIOTA, uint256 amount);
    event UnlockETH(address indexed user, uint256 amount);

    WIOTA public iotaToken;
    uint256 public lockedFunds;

    /*
    Set contract and ERC20/WIOTA owner
    */
    constructor(address _iotaToken) 
    Ownable(msg.sender)
    {
        iotaToken = WIOTA(_iotaToken);
    }

    /*
    Lock ETH funds 
    and emit an event that triggers the minting of a corresponding amount of wETH  
    to be sent to the 'addressIOTA' address on the other chain 
    */
    function lockETH(string calldata addressIOTA) external payable {
        require(msg.value > 0, "Amount zero or less");

        lockedFunds += msg.value;
        emit LockETH(msg.sender, addressIOTA, msg.value);
    }

    /*
    Mint 'amount' of wIOTA and transfer them to the 'recipient' address, only owner
    */
    function mintWIOTA(address recipient, uint256 amount) external onlyOwner {
        require(amount > 0, "Amount zero or less");

        iotaToken.mint(recipient, amount);
        emit MintWIOTA(recipient, amount);
    }

    /*

    // Burn 'amount' of wIOTA
    // and emit an event that triggers the unlocking of a corresponding amount of IOTA  
    // to be sent to the 'addressIOTA' address on the other chain
    */
    function burnWIOTA(uint256 amount, string calldata addressIOTA) external {
        require(amount > 0, "Amount zero or less");
        require(iotaToken.balanceOf(msg.sender) >= amount, "Insufficient IOTA funds");

        iotaToken.burnFrom(msg.sender, amount);
        emit BurnWIOTA(msg.sender, addressIOTA, amount);
    }

    /*
    Unlock 'amount' of ETH and transfer them to 'recipient'
    */
    function unlockETH(address payable recipient, uint256 amount) external onlyOwner {
        require(amount > 0, "Amount zero or less");
        require(lockedFunds >= amount, "Insufficient locked funds");

        lockedFunds -= amount;
        (bool success, ) = recipient.call{value: amount}("");
        require(success, "Failed to send Ether");
        emit UnlockETH(recipient, amount);
    }
}
