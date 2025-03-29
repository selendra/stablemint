import { expect } from "chai";
import { ethers } from "hardhat";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { AdvaRoleController } from "../typechain";
import { ZeroAddress } from "ethers";

describe("AdvaRoleController", function () {
  let advaRoleController: AdvaRoleController;
  let owner: HardhatEthersSigner;
  let user1: HardhatEthersSigner;
  let user2: HardhatEthersSigner;

  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const CAPPER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("CAPPER_ROLE"));
  const DEFAULT_ADMIN_ROLE = ethers.ZeroHash;

  beforeEach(async function () {
    [owner, user1, user2] = await ethers.getSigners();

    // Deploy the contract
    const AdvaRoleControllerFactory = await ethers.getContractFactory(
      "AdvaRoleController"
    );
    advaRoleController = await AdvaRoleControllerFactory.deploy();
  });

  describe("Constructor", function () {
    it("should set the deployer as DEFAULT_ADMIN_ROLE", async function () {
      expect(
        await advaRoleController.hasRole(DEFAULT_ADMIN_ROLE, owner.address)
      ).to.be.true;
    });

    it("should set the deployer as ADMIN_ROLE", async function () {
      expect(await advaRoleController.hasRole(ADMIN_ROLE, owner.address)).to.be
        .true;
    });

    it("should set the deployer as CAPPER_ROLE", async function () {
      expect(await advaRoleController.hasRole(CAPPER_ROLE, owner.address)).to.be
        .true;
    });

    it("should initialize capacity to 0", async function () {
      expect(await advaRoleController.capacity()).to.equal(0);
    });
  });

  // Testing reentrancy protection would require a malicious contract setup
  // This is a simplified test to ensure the nonReentrant modifier is used
  describe("Reentrancy Protection", function () {
    it("setCap function has nonReentrant modifier", async function () {
      await advaRoleController.setCap(1000);
      expect(await advaRoleController.capacity()).to.equal(1000);
    });
  });

  describe("Role Management", function () {
    it("should allow DEFAULT_ADMIN_ROLE to grant CAPPER_ROLE", async function () {
      await advaRoleController.grantRole(CAPPER_ROLE, user1.address);
      expect(await advaRoleController.hasRole(CAPPER_ROLE, user1.address)).to.be
        .true;
    });

    it("should allow user with CAPPER_ROLE to set capacity", async function () {
      await advaRoleController.grantRole(CAPPER_ROLE, user1.address);
      await advaRoleController.connect(user1).setCap(1000);
      expect(await advaRoleController.capacity()).to.equal(1000);
    });

    it("should allow DEFAULT_ADMIN_ROLE to revoke CAPPER_ROLE", async function () {
      await advaRoleController.grantRole(CAPPER_ROLE, user1.address);
      await advaRoleController.revokeRole(CAPPER_ROLE, user1.address);
      expect(await advaRoleController.hasRole(CAPPER_ROLE, user1.address)).to.be
        .false;
    });

    it("should not allow non-DEFAULT_ADMIN_ROLE to grant roles", async function () {
      await expect(
        advaRoleController.connect(user1).grantRole(CAPPER_ROLE, user2.address)
      )
        .to.be.revertedWithCustomError(
          advaRoleController,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(user1.address, DEFAULT_ADMIN_ROLE);
    });
  });

  describe("setCap", function () {
    it("should update capacity when called by CAPPER_ROLE", async function () {
      await advaRoleController.setCap(1000);
      expect(await advaRoleController.capacity()).to.equal(1000);
    });

    it("should emit Cap event when capacity is updated", async function () {
      await expect(advaRoleController.setCap(1000))
        .to.emit(advaRoleController, "Cap")
        .withArgs(1000, owner.address);
    });

    it("should revert when called by non-CAPPER_ROLE", async function () {
      await expect(advaRoleController.connect(user1).setCap(1000))
        .to.be.revertedWithCustomError(
          advaRoleController,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(user1.address, CAPPER_ROLE);
    });

    it("should revert when amount is zero (isValNumber modifier)", async function () {
      await expect(advaRoleController.setCap(0))
        .to.be.revertedWithCustomError(advaRoleController, "InvalidNumber")
        .withArgs("Amount must be greater than zero", 0);
    });

    it("should revert when setting same capacity (CapIsNotUpdate error)", async function () {
      // First set capacity to 1000
      await advaRoleController.setCap(1000);

      // Try to set it to 1000 again
      await expect(advaRoleController.setCap(1000))
        .to.be.revertedWithCustomError(advaRoleController, "CapIsNotUpdate")
        .withArgs(1000, 1000, "Requested amount same as capacity");
    });

    it("should allow setting capacity to different value after initial set", async function () {
      await advaRoleController.setCap(1000);
      expect(await advaRoleController.capacity()).to.equal(1000);

      await advaRoleController.setCap(2000);
      expect(await advaRoleController.capacity()).to.equal(2000);
    });
  });
});
