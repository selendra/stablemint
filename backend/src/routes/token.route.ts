import { Router } from "express";
import * as loyaltyController from "../controllers/loyalty.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { body } from "express-validator";

const router = Router();

router.get("/", authMiddleware, loyaltyController.getAllTokens);
router.get("/created", authMiddleware, loyaltyController.getAllCreatedTokens);
router.get("/pending", authMiddleware, loyaltyController.getAllPendingTokens);
router.get("/rejected", authMiddleware, loyaltyController.getAllRejectedTokens);

router.post(
	"/request",
	[
		body("name").isString().withMessage("name is required"),
		body("symbol").isString().withMessage("symbol is required"),
		body("stable_coin_amount")
			.isNumeric()
			.withMessage("stable_coin_amount is required"),
		body("ratio").isNumeric().withMessage("ratio is required"),
		authMiddleware,
	],
	loyaltyController.requestToken
);

router.post(
	"/reject",
	[
		body("token_id").isString().withMessage("token_id is required"),
		authMiddleware,
	],
	loyaltyController.rejectToken
);

export default router;
