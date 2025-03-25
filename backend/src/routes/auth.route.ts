import { Router } from "express";
import { body } from "express-validator";
import * as authController from "../controllers/auth.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

// // Register user
router.post(
	"/register",
	[
		body("name").not().isEmpty().withMessage("Name is required"),
		body("email").isEmail().withMessage("Please include a valid email"),
		body("password")
			.isLength({ min: 6 })
			.withMessage("Password must be at least 6 characters long"),
	],
	authController.register
);

// Login user
router.post(
	"/login",
	[
		body("email").isEmail().withMessage("Please include a valid email"),
		body("password").exists().withMessage("Password is required"),
	],
	authController.login
);

// Get current user (protected route)
router.get("/me", authMiddleware, authController.getCurrentUser);
router.get("/user/:userId", authMiddleware, authController.getUserById);

export default router;
