import { Request, Response } from "express";
import jwt from "jsonwebtoken";
import { validationResult } from "express-validator";
import User from "../models/user.model";
import Token from "../models/token.model";
import { comparePassword } from "../utils/password";
import config from "../config";
import { ethers } from "ethers";
import { DecodedToken } from "../middleware/auth.middleware";
import { getAuth, isAdmin } from "../utils/isAdmin";

export async function getAllTokens(req: Request, res: Response) {
	try {
		const data = await Token.find({});

		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllPendingTokens(req: Request, res: Response) {
	try {
		const data = await Token.find({ status: "PENDING" });

		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllRejectedTokens(req: Request, res: Response) {
	try {
		const data = await Token.find({ status: "REJECTED" });

		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllCreatedTokens(req: Request, res: Response) {
	try {
		const data = await Token.find({ status: "CREATED" });
		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getTokenById(req: Request, res: Response) {
	try {
		const data = await Token.findOne({ _id: req.body.token_id });
		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getTokensByOwnerId(req: Request, res: Response) {
	try {
		const data = await Token.find({ owner_id: req.body.owner_id });
		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function rejectToken(req: Request, res: Response) {
	try {
		// @ts-ignore
		const auth = getAuth(req);
		await isAdmin(auth.userId);

		const data = await Token.findOneAndUpdate(
			{ _id: req.body.token_id },
			{ status: "REJECTED" }
		);
		res.json(data);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function requestToken(req: Request, res: Response) {
	try {
		// @ts-ignore
		const user: DecodedToken = req?.user;

		const { name, symbol, stable_coin_amount, ratio } = req.body;
		const existing = await Token.find({
			$or: [{ name: name }, { symbol: symbol }],
		});

		if (existing.length > 0) {
			return res
				.status(400)
				.json({ message: "Token name or symbol is not available." });
		}

		const created = await new Token({
			name,
			symbol,
			stable_coin_amount,
			ratio,
			owner_id: user.userId,
			status: "PENDING",
			token_address: null,
			rejected_reason: null,
		}).save();

		res.json(created);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}
