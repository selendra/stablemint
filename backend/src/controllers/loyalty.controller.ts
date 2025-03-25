import { Request, Response } from "express";
import jwt from "jsonwebtoken";
import { validationResult } from "express-validator";
import User from "../models/user.model";
import Token from "../models/token.model";
import { comparePassword } from "../utils/password";
import config from "../config";
import { ethers } from "ethers";

export async function getAllTokens(req: Request, res: Response) {
	try {
		return await Token.find({});
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllPendingTokens(req: Request, res: Response) {
	try {
		return await Token.find({ status: "PENDING" });
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllRejectedTokens(req: Request, res: Response) {
	try {
		return await Token.find({ status: "REJECTED" });
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getAllCreatedTokens(req: Request, res: Response) {
	try {
		return await Token.find({ status: "CREATED" });
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getTokenById(req: Request, res: Response) {
	try {
		return await Token.findOne({ _id: req.body.token_id });
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function getTokensByOwnerId(req: Request, res: Response) {
	try {
		return await Token.find({ owner_id: req.body.owner_id });
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}

export async function rejectToken(req: Request, res: Response) {
	try {
		return await Token.findOneAndUpdate(
			{ _id: req.body.owner_id },
			{ status: "REJECTED" }
		);
	} catch (error) {
		console.error("getAllTokens error:", error);
		res.status(500).json({ message: "Server error" });
	}
}
