import { ethers } from "ethers";
import mongoose, { Document, Schema } from "mongoose";
import { hashPassword } from "../utils/password";

export interface IToken extends Document {
	name: string;
	symbol: string;
	token_address: string | null;
	owner_id: string;
	status: "PENDING" | "CREATED" | "REJECTED";
	rejected_reason: string | null;
	stable_coin_amount: number;
	ratio: number;
}

const tokenSchema = new Schema<IToken>(
	{
		name: {
			type: String,
			required: [true, "Name is required"],
			trim: true,
		},
		symbol: {
			type: String,
			required: [true, "Symbol is required"],
			trim: true,
		},
		token_address: {
			type: String,
			trim: true,
			default: null,
		},
		owner_id: {
			type: String,
			required: [true, "Owner ID is required"],
			trim: true,
		},
		status: {
			type: String,
			required: [true, "Owner ID is required"],
			trim: true,
			enum: ["PENDING", "CREATED", "REJECTED"],
		},
		rejected_reason: {
			type: String,
			default: null,
		},
		stable_coin_amount: {
			type: Number,
			required: [true, "stable_coin_amount is required"],
			min: 1,
		},
		ratio: {
			type: Number,
			required: [true, "ratio is required"],
		},
	},
	{
		timestamps: true,
	}
);

export default mongoose.model<IToken>("tokens", tokenSchema);
