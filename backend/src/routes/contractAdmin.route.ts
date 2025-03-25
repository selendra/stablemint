import express, { Request, Response } from "express";
import { body, param } from "express-validator";
import { validate } from "../error";
import { authMiddleware, DecodedToken } from "../middleware/auth.middleware";
import { adminContract, userContract } from "../config";
import * as authController from "../controllers/auth.controller";
import User from "../models/user.model";
import { ethers } from "ethers";
import { getAuth, isAdmin } from "../utils/isAdmin";
import Token from "../models/token.model";

// Initialize router
const router = express.Router();

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
			const auth = getAuth(req);
			await isAdmin(auth.userId);

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

// AddWhiteList
stableCoinRouter.post(
	"/whitelist/add",
	[body("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const auth = getAuth(req);
			await isAdmin(auth.userId);

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

// RemoveWhiteList
stableCoinRouter.post(
	"/whitelist/remove",
	[body("address").isString().withMessage("Valid address required"), validate],
	async (req: Request, res: Response) => {
		try {
			const auth = getAuth(req);
			await isAdmin(auth.userId);

			const result = await adminContract.removeFromWhitelist(req.body.address);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to add to whitelist",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

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

stableCoinRouter.post(
	"/transfer",
	[
		body("addresses").isString().withMessage("Address must be an string"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			// @ts-ignore
			const _user: DecodedToken = req?.user;

			if (!_user) {
				return res.status(401).json({
					message: "Unauthorized",
				});
			}

			const user = await User.findById(_user.userId);
			if (!user) {
				return res.status(404).json({ message: "User not found" });
			}

			const wallet = ethers.Wallet.fromPhrase(user.privateKey.toString());

			const result = await userContract(wallet.privateKey).transferStableCoin(
				req.body.addresses,
				Number(req.body.amount)
			);

			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to Transfer Stable coin",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// Token Factory Routes
const tokenFactoryRouter = express.Router();

// CreateLoyaltyToken
tokenFactoryRouter.post(
	"/create",
	[
		body("token_id").isString().withMessage("token_id is required"),
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
			const auth = getAuth(req);
			await isAdmin(auth.userId);

			const { name, symbol, tokenOwner, tokensPerStableCoin, token_id } =
				req.body;
			const tokenAddress = await adminContract.createToken(
				name,
				symbol,
				tokenOwner,
				Number(tokensPerStableCoin)
			);

			await Token.findOneAndUpdate(
				{ _id: token_id },
				{ status: "CREATED", token_address: tokenAddress }
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
			const auth = getAuth(req);
			await isAdmin(auth.userId);

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

tokenFactoryRouter.get(
	"/supply/:tokenAddress",
	[
		param("tokenAddress")
			.isString()
			.withMessage("Valid token address required"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const balance = await adminContract.checkTokenTotalSupply(
				req.params.tokenAddress
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

tokenFactoryRouter.post(
	"/transfer",
	[
		body("tokenAddress").isString().withMessage("Addresses must be an string"),
		body("toAddress").isString().withMessage("Addresses must be an string"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			// @ts-ignore
			const _user: DecodedToken = req?.user;

			if (!_user) {
				return res.status(401).json({
					message: "Unauthorized",
				});
			}

			const user = await User.findById(_user.userId);
			if (!user) {
				return res.status(404).json({ message: "User not found" });
			}

			const wallet = ethers.Wallet.fromPhrase(user.privateKey.toString());

			const { tokenAddress, toAddress, amount } = req.body;

			const result = await userContract(wallet.privateKey).tokenTransfer(
				tokenAddress,
				toAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to Transfer Token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

// StableCoin Routes
const swapperRouter = express.Router();

swapperRouter.post(
	"/swap_stable_coin_to_token",
	[
		body("tokenAddress").isString().withMessage("Valid token address required"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			// @ts-ignore
			const _user: DecodedToken = req?.user;

			if (!_user) {
				return res.status(401).json({
					message: "Unauthorized",
				});
			}

			const user = await User.findById(_user.userId);
			if (!user) {
				return res.status(404).json({ message: "User not found" });
			}

			const wallet = ethers.Wallet.fromPhrase(user.privateKey.toString());

			const { tokenAddress, amount } = req.body;
			const result = await userContract(wallet.privateKey).swapperStableToken(
				tokenAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to swap token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

swapperRouter.post(
	"/swap_token_to_stable_coin",
	[
		body("tokenAddress").isString().withMessage("Valid token address required"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			// @ts-ignore
			const _user: DecodedToken = req?.user;

			if (!_user) {
				return res.status(401).json({
					message: "Unauthorized",
				});
			}

			const user = await User.findById(_user.userId);
			if (!user) {
				return res.status(404).json({ message: "User not found" });
			}

			const { tokenAddress, amount } = req.body;
			const wallet = ethers.Wallet.fromPhrase(user.privateKey.toString());
			const result = await userContract(wallet.privateKey).swapperTokenStable(
				tokenAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to swap token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

const userRouter = express.Router();

userRouter.post(
	"/transfer",
	[
		body("toAddress").isString().withMessage("Addresses must be an string"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			// @ts-ignore
			const _user: DecodedToken = req?.user;

			if (!_user) {
				return res.status(401).json({
					message: "Unauthorized",
				});
			}

			const user = await User.findById(_user.userId);
			if (!user) {
				return res.status(404).json({ message: "User not found" });
			}

			const wallet = ethers.Wallet.fromPhrase(user.privateKey.toString());

			const { toAddress, amount } = req.body;

			const result = await userContract(wallet.privateKey).nativeTransfer(
				toAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to Transfer Native Token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

userRouter.post(
	"/airdrop",
	[
		body("toAddress").isString().withMessage("Addresses must be an string"),
		body("amount").isNumeric().withMessage("Amount must be a number"),
		validate,
	],
	async (req: Request, res: Response) => {
		try {
			const auth = getAuth(req);
			await isAdmin(auth.userId);

			const { toAddress, amount } = req.body;

			const result = await adminContract.nativeTransfer(
				toAddress,
				Number(amount)
			);
			res.json({ success: true, result });
		} catch (error) {
			res.status(500).json({
				message: "Failed to Transfer Native Token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

userRouter.get(
	"/sel/:address",
	validate,
	async (req: Request, res: Response) => {
		try {
			getAuth(req);
			const balance = await adminContract.getSel(req.params.address);
			res.json({ balance });
		} catch (error) {
			res.status(500).json({
				message: "Failed to Transfer Native Token",
				error: error instanceof Error ? error.message : String(error),
			});
		}
	}
);

userRouter.get("/", authMiddleware, authController.getAllUsers);

// Register all routers
router.use("/stablecoin", authMiddleware, stableCoinRouter);
router.use("/token", authMiddleware, tokenFactoryRouter);
router.use("/users", authMiddleware, userRouter);
router.use("/swapper", authMiddleware, swapperRouter);

export default router;
