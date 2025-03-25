"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.authMiddleware = void 0;
const jsonwebtoken_1 = __importDefault(require("jsonwebtoken"));
const config_1 = __importDefault(require("../config"));
const authMiddleware = (req, res, next) => {
    // Get token from header
    const token = req.header("Authorization")?.replace("Bearer ", "");
    // Check if token exists
    if (!token) {
        return res.status(401).json({ message: "No token, authorization denied" });
    }
    try {
        // Verify token
        const decoded = jsonwebtoken_1.default.verify(token, config_1.default.jwtSecret);
        // Add user from payload to request
        req.user = decoded;
        next();
    }
    catch (error) {
        res.status(401).json({ message: "Token is not valid" });
    }
};
exports.authMiddleware = authMiddleware;
