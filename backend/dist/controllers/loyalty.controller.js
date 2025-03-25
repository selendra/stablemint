"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAllTokens = getAllTokens;
exports.getAllPendingTokens = getAllPendingTokens;
exports.getAllRejectedTokens = getAllRejectedTokens;
exports.getAllCreatedTokens = getAllCreatedTokens;
exports.getTokenById = getTokenById;
exports.getTokensByOwnerId = getTokensByOwnerId;
exports.rejectToken = rejectToken;
exports.requestToken = requestToken;
const token_model_1 = __importDefault(require("../models/token.model"));
const isAdmin_1 = require("../utils/isAdmin");
async function getAllTokens(req, res) {
    try {
        return await token_model_1.default.find({});
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function getAllPendingTokens(req, res) {
    try {
        return await token_model_1.default.find({ status: "PENDING" });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function getAllRejectedTokens(req, res) {
    try {
        return await token_model_1.default.find({ status: "REJECTED" });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function getAllCreatedTokens(req, res) {
    try {
        return await token_model_1.default.find({ status: "CREATED" });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function getTokenById(req, res) {
    try {
        return await token_model_1.default.findOne({ _id: req.body.token_id });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function getTokensByOwnerId(req, res) {
    try {
        return await token_model_1.default.find({ owner_id: req.body.owner_id });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function rejectToken(req, res) {
    try {
        // @ts-ignore
        const auth = (0, isAdmin_1.getAuth)(req);
        await (0, isAdmin_1.isAdmin)(auth.userId);
        return await token_model_1.default.findOneAndUpdate({ _id: req.body.token_id }, { status: "REJECTED" });
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
async function requestToken(req, res) {
    try {
        // @ts-ignore
        const user = req?.user;
        const { name, symbol, owner_id, stable_coin_amount, ratio } = req.body;
        const existing = await token_model_1.default.find({
            $or: [{ name: name }, { symbol: symbol }],
        });
        if (existing.length > 0) {
            return res
                .status(400)
                .json({ message: "Token name or symbol is not available." });
        }
        return await new token_model_1.default({
            name,
            symbol,
            stable_coin_amount,
            ratio,
            owner_id: user.userId,
            status: "PENDING",
            token_address: null,
            rejected_reason: null,
        }).save();
    }
    catch (error) {
        console.error("getAllTokens error:", error);
        res.status(500).json({ message: "Server error" });
    }
}
