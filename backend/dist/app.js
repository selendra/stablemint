"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const mongoose_1 = __importDefault(require("mongoose"));
const cors_1 = __importDefault(require("cors"));
const config_1 = __importDefault(require("./config"));
const auth_route_1 = __importDefault(require("./routes/auth.route"));
const contractAdmin_route_1 = __importDefault(require("./routes/contractAdmin.route"));
const token_route_1 = __importDefault(require("./routes/token.route"));
// Initialize Express app
const app = (0, express_1.default)();
// Connect to MongoDB
mongoose_1.default
    .connect(config_1.default.mongodbUri)
    .then(() => {
    console.log("Connected to MongoDB");
})
    .catch((err) => {
    console.error("MongoDB connection error:", err);
    process.exit(1);
});
// Middleware
app.use((0, cors_1.default)());
app.use(express_1.default.json());
// Routes
app.use("/api/auth", auth_route_1.default);
// Routes
app.use("/api/contract", contractAdmin_route_1.default);
app.use("/api/tokens", token_route_1.default);
// Home route
app.get("/", (req, res) => {
    res.json({ message: "Welcome to the TypeScript Express Authentication API" });
});
// Error handling middleware
app.use((err, req, res, next) => {
    console.error(err.stack);
    res.status(500).json({
        message: "An unexpected error occurred",
        error: process.env.NODE_ENV === "development" ? err.message : undefined,
    });
});
exports.default = app;
