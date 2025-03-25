import express, { Request, Response } from "express";
import { body, param } from "express-validator";
import { validate } from "../error";
import { authMiddleware } from "../middleware/auth.middleware";
import { adminContract, userContract } from "../config";

// Initialize router
const router = express.Router();

// Get signer address
router.get("/address", authMiddleware, async (req: Request, res: Response) => {
  try {
    const address = await adminContract.getSignerAddress();
    res.json({ address });
  } catch (error) {
    res.status(500).json({
      message: "Failed to get signer address",
      error: error instanceof Error ? error.message : String(error),
    });
  }
});

// StableCoin Routes
const stableCoinRouter = express.Router();

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

stableCoinRouter.post(
  "/whitelist/batch-add",
  [
    body("addresses").isArray().withMessage("Addresses must be an array"),
    validate,
  ],
  async (req: Request, res: Response) => {
    try {
      const result = await adminContract.addBatchToWhitelist(
        req.body.addresses
      );
      res.json({ success: true, result });
    } catch (error) {
      res.status(500).json({
        message: "Failed to add batch to whitelist",
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
);

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
    body("private_key").isString().withMessage("Addresses must be an array"),
    body("addresses").isString().withMessage("Addresses must be an array"),
    body("amount").isNumeric().withMessage("Amount must be a number"),
    validate,
  ],
  async (req: Request, res: Response) => {
    try {
      const result = await userContract(
        req.body.private_key
      ).transferStableCoin(req.body.addresses, Number(req.body.amount));
      res.json({ success: true, result });
    } catch (error) {
      res.status(500).json({
        message: "Failed to add batch to whitelist",
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
);

// Token Factory Routes
const tokenFactoryRouter = express.Router();

tokenFactoryRouter.get(
  "/is-created/:address",
  [
    param("address").isString().withMessage("Valid token address required"),
    validate,
  ],
  async (req: Request, res: Response) => {
    try {
      const isCreated = await adminContract.isTokenCreatedByFactory(
        req.params.address
      );
      res.json({ isCreated });
    } catch (error) {
      res.status(500).json({
        message: "Failed to check token creation status",
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
);

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
    body("private_key").isArray().withMessage("Addresses must be an array"),
    body("tokenAddress").isArray().withMessage("Addresses must be an array"),
    body("addresses").isArray().withMessage("Addresses must be an array"),
    body("amount").isNumeric().withMessage("Amount must be a number"),
    validate,
  ],
  async (req: Request, res: Response) => {
    try {
      const { tokenAddress, addresses, amount } = req.body;

      const result = await userContract(req.body.private_key).tokenTransfer(
        tokenAddress,
        addresses,
        Number(amount)
      );
      res.json({ success: true, result });
    } catch (error) {
      res.status(500).json({
        message: "Failed to add batch to whitelist",
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
    body("privateKey").isString().withMessage("Valid token address required"),
    body("tokenAddress").isString().withMessage("Valid token address required"),
    body("amount").isNumeric().withMessage("Amount must be a number"),
    validate,
  ],
  async (req: Request, res: Response) => {
    try {
      const { tokenAddress, amount, privateKey } = req.body;
      const result = await userContract(privateKey).swapperStableToken(
        tokenAddress,
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

// Register all routers
router.use("/stablecoin", authMiddleware, stableCoinRouter);
router.use("/token", authMiddleware, tokenFactoryRouter);
router.use("/swapper", authMiddleware, swapperRouter);

export default router;
