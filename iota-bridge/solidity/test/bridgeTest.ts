import {
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { expect } from "chai";
import hre from "hardhat" ;

describe("Bridge", function () {
  async function deployAndSetBridge() {
    
    const [deployer, user] = await hre.ethers.getSigners();

    const IotaToken = await hre.ethers.getContractFactory("WIOTA");
    const iotaToken = await IotaToken.connect(deployer).deploy();

    const Bridge = await hre.ethers.getContractFactory("Bridge");
    const bridge = await Bridge.connect(deployer).deploy(iotaToken.target);

    await iotaToken.connect(deployer).transferOwnership(bridge.target);

    return { bridge , deployer, user, iotaToken};
  }

  it("Should deploy the contracts", async function () {
    const { bridge, iotaToken, deployer } = await loadFixture(deployAndSetBridge);

    expect(bridge.target).to.be.properAddress;
    expect(iotaToken.target).to.be.properAddress;

    expect(await bridge.owner()).to.equal(deployer.address);
  });

  it("Should allow a user to lock ETH", async function () {
    const { bridge, user} = await loadFixture(deployAndSetBridge);

    const lockAmount = hre.ethers.parseEther("1.0");
    const iotaAddress = "0xCAFE";
    
    
    await expect(bridge.connect(user).lockETH(iotaAddress, { value: lockAmount }))
      .to.emit(bridge, "LockETH")
      .withArgs(user.address, iotaAddress, lockAmount);

    expect(await hre.ethers.provider.getBalance(bridge.target)).to.equal(lockAmount);
    expect(await bridge.lockedFunds()).to.equal(lockAmount);
  });
  
  it("Should allow the deployer to mint wIOTA", async function () {
    const { bridge, deployer, user, iotaToken} = await loadFixture(deployAndSetBridge);

    const mintAmount = 100;
    
    expect(await bridge.connect(deployer).mintWIOTA(user.address, mintAmount))
      .to.emit(bridge, "MintIOTA")
      .withArgs(user.address, mintAmount);

    expect(await iotaToken.balanceOf(user.address)).to.equal(mintAmount);

    await expect(bridge.connect(user).mintWIOTA(user.address, mintAmount))
      .to.be.revertedWithCustomError(bridge, "OwnableUnauthorizedAccount");
  });

  it("Should allow a user to burn WIOTA", async function () {
    const { bridge, iotaToken, deployer, user } = await loadFixture(deployAndSetBridge);

    const mintAmount = 100;
    const burnAmount = 50;
    const iotaAddress = "0xCAFE";

    await bridge.connect(deployer).mintWIOTA(user.address, mintAmount);
    await iotaToken.connect(user).approve(bridge.target, burnAmount);
    
    expect(await bridge.connect(user).burnWIOTA(burnAmount, iotaAddress))
      .to.emit(bridge, "BurnIOTA")
      .withArgs(user.address, iotaAddress, burnAmount);

    expect(await iotaToken.balanceOf(user.address)).to.equal(mintAmount - burnAmount);
  });

  it("Should allow the owner to unlock ETH", async function () {
    const { bridge, deployer, user } = await loadFixture(deployAndSetBridge);

    const lockAmount = hre.ethers.parseEther("2.0");
    const unlockAmount = hre.ethers.parseEther("1.0"); 

    await bridge.connect(user).lockETH("0xCAFE", { value: lockAmount });

    
    expect(await bridge.connect(deployer).unlockETH(user.address, unlockAmount))
      .to.emit(bridge, "UnlockETH")
      .withArgs(user.address, unlockAmount);

    expect(await bridge.lockedFunds()).to.equal(lockAmount -unlockAmount);

    expect(await hre.ethers.provider.getBalance(bridge.target)).to.equal(lockAmount -unlockAmount);
  });
});
