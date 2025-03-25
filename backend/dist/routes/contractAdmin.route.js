"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const express_validator_1 = require("express-validator");
const error_1 = require("../error");
const auth_middleware_1 = require("../middleware/auth.middleware");
const config_1 = require("../config");
const authController = __importStar(require("../controllers/auth.controller"));
const user_model_1 = __importDefault(require("../models/user.model"));
const ethers_1 = require("ethers");
const isAdmin_1 = require("../utils/isAdmin");
// Initialize router
const router = express_1.default.Router();
// StableCoin Routes
const stableCoinRouter = express_1.default.Router();
// checkAnyBalance
stableCoinRouter.get("/balance/:address", [(0, express_validator_1.param)("address").isString().withMessage("Valid address required"), error_1.validate], async (req, res) => {
    try {
        const balance = await config_1.adminContract.checkBalance(req.params.address);
        res.json({ balance });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to check balance",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// getStableCoinTotalSupply
stableCoinRouter.get("/total-supply", async (req, res) => {
    try {
        const totalSupply = await config_1.adminContract.checkTotalSupply();
        res.json({ totalSupply });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to check total supply",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// minStableCoin
stableCoinRouter.post("/mint", [
    (0, express_validator_1.body)("toAddress")
        .isString()
        .withMessage("Valid recipient address required"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        const { toAddress, amount } = req.body;
        const result = await config_1.adminContract.mintStableCoin(toAddress, Number(amount));
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to mint StableCoin",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// withdrawMoney
stableCoinRouter.post("/withdraw", [
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    (0, express_validator_1.body)("withdrawerAddress")
        .isString()
        .withMessage("Valid withdrawer address required"),
    (0, express_validator_1.body)("reason").isString().withMessage("Reason is required"),
    error_1.validate,
], async (req, res) => {
    try {
        const { amount, withdrawerAddress, reason } = req.body;
        const result = await config_1.adminContract.withdrawStableCoin(Number(amount), withdrawerAddress, reason);
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to withdraw StableCoin",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// AddWhiteList
stableCoinRouter.post("/whitelist/add", [(0, express_validator_1.body)("address").isString().withMessage("Valid address required"), error_1.validate], async (req, res) => {
    try {
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        const result = await config_1.adminContract.addToWhitelist(req.body.address);
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to add to whitelist",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// RemoveWhiteList
stableCoinRouter.post("/whitelist/remove", [(0, express_validator_1.body)("address").isString().withMessage("Valid address required"), error_1.validate], async (req, res) => {
    try {
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        const result = await config_1.adminContract.removeFromWhitelist(req.body.address);
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to add to whitelist",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// getAccountWhitelistStatus
stableCoinRouter.get("/whitelist/:address", [(0, express_validator_1.param)("address").isString().withMessage("Valid address required"), error_1.validate], async (req, res) => {
    try {
        const isWhitelisted = await config_1.adminContract.checkWhitelist(req.params.address);
        res.json({ isWhitelisted });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to check whitelist status",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
stableCoinRouter.post("/transfer", [
    (0, express_validator_1.body)("addresses").isString().withMessage("Addresses must be an array"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        // @ts-ignore
        const _user = req?.user;
        if (!_user) {
            return res.status(401).json({
                message: "Unauthorized",
            });
        }
        const user = await user_model_1.default.findById(_user.userId);
        if (!user) {
            return res.status(404).json({ message: "User not found" });
        }
        const wallet = ethers_1.ethers.Wallet.fromPhrase(user.privateKey.toString());
        const result = await (0, config_1.userContract)(wallet.privateKey).transferStableCoin(req.body.addresses, Number(req.body.amount));
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to Transfer Stable coin",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// Token Factory Routes
const tokenFactoryRouter = express_1.default.Router();
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
tokenFactoryRouter.post("/create", [
    (0, express_validator_1.body)("name").isString().withMessage("Token name is required"),
    (0, express_validator_1.body)("symbol").isString().withMessage("Token symbol is required"),
    (0, express_validator_1.body)("tokenOwner")
        .isString()
        .withMessage("Token owner address is required"),
    (0, express_validator_1.body)("tokensPerStableCoin")
        .isNumeric()
        .withMessage("tokensPerStableCoin must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        const { name, symbol, tokenOwner, tokensPerStableCoin } = req.body;
        const tokenAddress = await config_1.adminContract.createToken(name, symbol, tokenOwner, Number(tokensPerStableCoin));
        res.json({ success: true, tokenAddress });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to create token",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// MintLoyaltyToken
tokenFactoryRouter.post("/mint", [
    (0, express_validator_1.body)("tokenAddress").isString().withMessage("Valid token address required"),
    (0, express_validator_1.body)("toAddress")
        .isString()
        .withMessage("Valid recipient address required"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        const { tokenAddress, toAddress, amount } = req.body;
        const result = await config_1.adminContract.mintToken(tokenAddress, toAddress, Number(amount));
        res.json(result);
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to mint token",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// GetAllLoyaltyTokens
tokenFactoryRouter.get("/all", async (req, res) => {
    try {
        const tokens = await config_1.adminContract.getAllCreatedTokens();
        res.json({ tokens });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to get all created tokens",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// GetLoyaltyTokenBalance
tokenFactoryRouter.get("/balance/:tokenAddress/:accountAddress", [
    (0, express_validator_1.param)("tokenAddress")
        .isString()
        .withMessage("Valid token address required"),
    (0, express_validator_1.param)("accountAddress")
        .isString()
        .withMessage("Valid account address required"),
    error_1.validate,
], async (req, res) => {
    try {
        const balance = await config_1.adminContract.checkTokenBalance(req.params.tokenAddress, req.params.accountAddress);
        res.json({ balance });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to check token balance",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
tokenFactoryRouter.get("/supply/:tokenAddress", [
    (0, express_validator_1.param)("tokenAddress")
        .isString()
        .withMessage("Valid token address required"),
    error_1.validate,
], async (req, res) => {
    try {
        const balance = await config_1.adminContract.checkTokenTotalSupply(req.params.tokenAddress);
        res.json({ balance });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to check token balance",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
tokenFactoryRouter.post("/transfer", [
    (0, express_validator_1.body)("tokenAddress").isString().withMessage("Addresses must be an string"),
    (0, express_validator_1.body)("toAddress").isString().withMessage("Addresses must be an string"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        // @ts-ignore
        const _user = req?.user;
        if (!_user) {
            return res.status(401).json({
                message: "Unauthorized",
            });
        }
        const user = await user_model_1.default.findById(_user.userId);
        if (!user) {
            return res.status(404).json({ message: "User not found" });
        }
        const wallet = ethers_1.ethers.Wallet.fromPhrase(user.privateKey.toString());
        const { tokenAddress, toAddress, amount } = req.body;
        const result = await (0, config_1.userContract)(wallet.privateKey).tokenTransfer(tokenAddress, toAddress, Number(amount));
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to Transfer Token",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
// StableCoin Routes
const swapperRouter = express_1.default.Router();
swapperRouter.post("/swap_stable_coin_to_token", [
    (0, express_validator_1.body)("tokenAddress").isString().withMessage("Valid token address required"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        // @ts-ignore
        const _user = req?.user;
        if (!_user) {
            return res.status(401).json({
                message: "Unauthorized",
            });
        }
        const user = await user_model_1.default.findById(_user.userId);
        if (!user) {
            return res.status(404).json({ message: "User not found" });
        }
        const wallet = ethers_1.ethers.Wallet.fromPhrase(user.privateKey.toString());
        const { tokenAddress, amount } = req.body;
        const result = await (0, config_1.userContract)(wallet.privateKey).swapperStableToken(tokenAddress, Number(amount));
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to swap token",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
swapperRouter.post("/swap_token_to_stable_coin", [
    (0, express_validator_1.body)("tokenAddress").isString().withMessage("Valid token address required"),
    (0, express_validator_1.body)("amount").isNumeric().withMessage("Amount must be a number"),
    error_1.validate,
], async (req, res) => {
    try {
        // @ts-ignore
        const _user = req?.user;
        if (!_user) {
            return res.status(401).json({
                message: "Unauthorized",
            });
        }
        const user = await user_model_1.default.findById(_user.userId);
        if (!user) {
            return res.status(404).json({ message: "User not found" });
        }
        const { tokenAddress, amount } = req.body;
        const wallet = ethers_1.ethers.Wallet.fromPhrase(user.privateKey.toString());
        const result = await (0, config_1.userContract)(wallet.privateKey).swapperTokenStable(tokenAddress, Number(amount));
        res.json({ success: true, result });
    }
    catch (error) {
        res.status(500).json({
            message: "Failed to swap token",
            error: error instanceof Error ? error.message : String(error),
        });
    }
});
const userRouter = express_1.default.Router();
userRouter.get("/", auth_middleware_1.authMiddleware, authController.getAllUsers);
// Register all routers
router.use("/stablecoin", auth_middleware_1.authMiddleware, stableCoinRouter);
router.use("/token", auth_middleware_1.authMiddleware, tokenFactoryRouter);
router.use("/users", userRouter);
router.use("/swapper", auth_middleware_1.authMiddleware, swapperRouter);
exports.default = router;
