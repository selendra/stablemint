import express, { Request, Response } from "express";
import { body, param, validationResult } from "express-validator";
import jwt from "jsonwebtoken";
import { Secret } from "jsonwebtoken";
import { Admin } from "../utils/contract";
import { validate } from "../error";
import { authMiddleware } from "../middleware/auth.middleware";
import { adminContract } from "../config";
import * as authController from "../controllers/auth.controller";

// Initialize router
const router = express.Router();

// Get signer address
// router.get("/address", authMiddleware, async (req: Request, res: Response) => {
//   try {
//     const address = await adminContract.getSignerAddress();
//     res.json({ address });
//   } catch (error) {
//     res.status(500).json({
//       message: "Failed to get signer address",
//       error: error instanceof Error ? error.message : String(error),
//     });
//   }
// });

// Role Management Routes
const roleRouter = express.Router();

// roleRouter.post(
// 	"/grant",
// 	[
// 		body("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		body("role").isString().withMessage("Role name is required"),
// 		body("accountAddress")
// 			.isString()
// 			.withMessage("Account address is required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const { contractAddress, role, accountAddress } = req.body;
// 			const result = await adminContract.grantRole(
// 				contractAddress,
// 				role,
// 				accountAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to grant role",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// roleRouter.post(
// 	"/revoke",
// 	[
// 		body("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		body("role").isString().withMessage("Role name is required"),
// 		body("accountAddress")
// 			.isString()
// 			.withMessage("Account address is required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const { contractAddress, role, accountAddress } = req.body;
// 			const result = await adminContract.revokeRole(
// 				contractAddress,
// 				role,
// 				accountAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to revoke role",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// roleRouter.get(
// 	"/check",
// 	[
// 		body("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		body("role").isString().withMessage("Role name is required"),
// 		body("accountAddress")
// 			.isString()
// 			.withMessage("Account address is required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const { contractAddress, role, accountAddress } = req.body;
// 			const hasRole = await adminContract.hasRole(
// 				contractAddress,
// 				role,
// 				accountAddress
// 			);
// 			res.json({ hasRole });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to check role",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// Pause/Unpause Routes
const pauseRouter = express.Router();

// pauseRouter.get(
// 	"/status/:contractAddress",
// 	[
// 		param("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const isPaused = await adminContract.isPausedContract(
// 				req.params.contractAddress
// 			);
// 			res.json({ isPaused });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to check pause status",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// pauseRouter.post(
// 	"/pause",
// 	[
// 		body("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const result = await adminContract.pauseContract(
// 				req.body.contractAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to pause contract",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// pauseRouter.post(
// 	"/unpause",
// 	[
// 		body("contractAddress")
// 			.isString()
// 			.withMessage("Valid contract address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const result = await adminContract.unpauseContract(
// 				req.body.contractAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to unpause contract",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// StableCoin Routes
const stableCoinRouter = express.Router();

// checkAnyBalance
stableCoinRouter.get(
	"/balance/:address",
	[param("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const balance = await adminContract.checkBalance(req.params.address);
			res.json({ balance });
		} catch (error) {
			res.status(500).json({
				message: "Failed to check balance",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// getStableCoinTotalSupply
stableCoinRouter.get("/total-supply", async (req: Request, res: Response) => {
	try {
		const totalSupply = await adminContract.checkTotalSupply();
		res.json({ totalSupply });
	} catch (error) {
		res.status(500).json({
			message: "Failed to check total supply",
			error: error instanceof Error ? error.message : String(error),
		});
	}
});

// minStableCoin
stableCoinRouter.post(
	"/mint",
	[
		body("toAddress")
			.isString()
			.withMessage("Valid recipient address required"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const { toAddress, amount } = req.body;
			const result = await adminContract.mintStableCoin(
				toAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to mint StableCoin",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// withdrawMoney
stableCoinRouter.post(
	"/withdraw",
	[
		body("amount").isNumeric().withMessage("Amount must be a number"),
		body("withdrawerAddress")
			.isString()
			.withMessage("Valid withdrawer address required"),
		body("reason").isString().withMessage("Reason is required"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const { amount, withdrawerAddress, reason } = req.body;
			const result = await adminContract.withdrawStableCoin(
				Number(amount),
				withdrawerAddress,
				reason
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to withdraw StableCoin",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// stableCoinRouter.post(
// 	"/whitelist/policy",
// 	[
// 		body("enforce").isBoolean().withMessage("Enforce must be boolean"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const { enforce } = req.body;
// 			const result = await adminContract.setWhitelistReceiverPolicy(enforce);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to set whitelist policy",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// AddWhiteList
stableCoinRouter.post(
	"/whitelist/add",
	[body("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const result = await adminContract.addToWhitelist(req.body.address);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to add to whitelist",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// stableCoinRouter.post(
// 	"/whitelist/batch-add",
// 	[
// 		body("addresses").isArray().withMessage("Addresses must be an array"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const result = await adminContract.addBatchToWhitelist(
// 				req.body.addresses
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to add batch to whitelist",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// RemoveWhiteList
stableCoinRouter.post(
	"/whitelist/remove",
	[body("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const result = await adminContract.removeFromWhitelist(req.body.address);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to remove from whitelist",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// stableCoinRouter.get(
// 	"/whitelist/enforce",
// 	async (req: Request, res: Response) => {
// 		try {
// 			const enforced = await adminContract.checkEnforceWhitelist();
// 			res.json({ enforced });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to check whitelist enforcement",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// getAccountWhitelistStatus
stableCoinRouter.get(
	"/whitelist/:address",
	[param("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const isWhitelisted = await adminContract.checkWhitelist(
				req.params.address
			);
			res.json({ isWhitelisted });
		} catch (error) {
			res.status(500).json({
				message: "Failed to check whitelist status",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// Token Factory Routes
const tokenFactoryRouter = express.Router();

// createToken
// tokenFactoryRouter.get(
// 	"/is-created/:address",
// 	[
// 		param("address").isString().withMessage("Valid token address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const isCreated = await adminContract.isTokenCreatedByFactory(
// 				req.params.address
// 			);
// 			res.json({ isCreated });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to check token creation status",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// CreateLoyaltyToken
tokenFactoryRouter.post(
	"/create",
	[
		body("name").isString().withMessage("Token name is required"),
		body("symbol").isString().withMessage("Token symbol is required"),
		body("tokenOwner")
			.isString()
			.withMessage("Token owner address is required"),
		body("tokensPerStableCoin")
			.isNumeric()
			.withMessage("tokensPerStableCoin must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const { name, symbol, tokenOwner, tokensPerStableCoin } = req.body;
			const tokenAddress = await adminContract.createToken(
				name,
				symbol,
				tokenOwner,
				Number(tokensPerStableCoin)
			);
			res.json({ success: true, tokenAddress });
		} catch (error) {
			res.status(500).json({
				message: "Failed to create token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// MintLoyaltyToken
tokenFactoryRouter.post(
	"/mint",
	[
		body("tokenAddress").isString().withMessage("Valid token address required"),
		body("toAddress")
			.isString()
			.withMessage("Valid recipient address required"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const { tokenAddress, toAddress, amount } = req.body;
			const result = await adminContract.mintToken(
				tokenAddress,
				toAddress,
				Number(amount)
			);
			res.json(result);
		} catch (error) {
			res.status(500).json({
				message: "Failed to mint token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// GetAllLoyaltyTokens
tokenFactoryRouter.get("/all", async (req: Request, res: Response) => {
	try {
		const tokens = await adminContract.getAllCreatedTokens();
		res.json({ tokens });
	} catch (error) {
		res.status(500).json({
			message: "Failed to get all created tokens",
			error: error instanceof Error ? error.message : String(error),
		});
	}
});

// GetLoyaltyTokenBalance
tokenFactoryRouter.get(
	"/balance/:tokenAddress/:accountAddress",
	[
		param("tokenAddress")
			.isString()
			.withMessage("Valid token address required"),
		param("accountAddress")
			.isString()
			.withMessage("Valid account address required"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const balance = await adminContract.checkTokenBalance(
				req.params.tokenAddress,
				req.params.accountAddress
			);
			res.json({ balance });
		} catch (error) {
			res.status(500).json({
				message: "Failed to check token balance",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);
// Token Swap Routes
const tokenSwapRouter = express.Router();

// tokenSwapRouter.post(
// 	"/set-fee-percentage",
// 	[
// 		body("feePercentage")
// 			.isNumeric()
// 			.withMessage("Fee percentage must be a number"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const result = await adminContract.setFeePercentage(
// 				Number(req.body.feePercentage)
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to set fee percentage",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// tokenSwapRouter.post(
// 	"/set-fee-collector",
// 	[
// 		body("feeCollectorAddress")
// 			.isString()
// 			.withMessage("Valid fee collector address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const result = await adminContract.setFeeCollector(
// 				req.body.feeCollectorAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to set fee collector",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

// tokenSwapRouter.post(
// 	"/emergency-withdraw",
// 	[
// 		body("tokenAddress").isString().withMessage("Valid token address required"),
// 		body("amount").isNumeric().withMessage("Amount must be a number"),
// 		body("toAddress")
// 			.isString()
// 			.withMessage("Valid recipient address required"),
// 		validate,
// 	],
// 	async (req: Request, res: Response) => {
// 		try {
// 			const { tokenAddress, amount, toAddress } = req.body;
// 			const result = await adminContract.emergencyWithdraw(
// 				tokenAddress,
// 				Number(amount),
// 				toAddress
// 			);
// 			res.json({ success: true, result });
// 		} catch (error) {
// 			res.status(500).json({
// 				message: "Failed to execute emergency withdrawal",
// 				error: error instanceof Error ? error.message : String(error),
// 			});
// 		}
// 	}
// );

const userRouter = express.Router();

userRouter.get("/", authMiddleware, authController.getAllUsers);

// Register all routers
router.use("/stablecoin", authMiddleware, stableCoinRouter);
router.use("/token", authMiddleware, tokenFactoryRouter);
router.use("/users", userRouter);
// router.use("/role", authMiddleware, roleRouter);
// router.use("/contract", authMiddleware, pauseRouter);
// router.use("/swap", authMiddleware, tokenSwapRouter);

export default router;
