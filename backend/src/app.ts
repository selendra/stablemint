import express from "express";
import mongoose from "mongoose";
import cors from "cors";
import config from "./config";
import authRoutes from "./routes/auth.route";
import adminContractRoutes from "./routes/contractAdmin.route";
import tokenRoutes from "./routes/token.route";

// Initialize Express app
const app = express();

// Connect to MongoDB
mongoose
	.connect(config.mongodbUri)
	.then(() => {
		console.log("Connected to MongoDB");
	})
	.catch((err) => {
		console.error("MongoDB connection error:", err);
		process.exit(1);
	});

// Middleware
app.use(cors());
app.use(express.json());

// Routes
app.use("/api/auth", authRoutes);
// Routes
app.use("/api/contract", adminContractRoutes);
app.use("/api/tokens", tokenRoutes);

// Home route
app.get("/", (req, res) => {
	res.json({ message: "Welcome to the TypeScript Express Authentication API" });
});

// Error handling middleware
app.use(
	(
		err: any,
		req: express.Request,
		res: express.Response,
		next: express.NextFunction
	) => {
		console.error(err.stack);
		res.status(500).json({
			message: "An unexpected error occurred",
			error: process.env.NODE_ENV === "development" ? err.message : undefined,
		});
	}
);

export default app;
