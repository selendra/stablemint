import dotenv from "dotenv";
import { Admin } from "../utils/contract/admin";

dotenv.config();

export default {
	port: process.env.PORT || 5000,
	mongodbUri: process.env.MONGODB_URI || "mongodb://localhost:27017/stablemint",
	jwtSecret: process.env.JWT_SECRET || "default_jwt_secret",
	jwtExpiresIn: process.env.JWT_EXPIRES_IN || "7d",
	contract: {
		stableCoinAdress: "0xffFEdB07dbc5A93A3c7653930e46Bd9332468559",
		tokenFactoryAddress: "0x376e34036b77704B7558Dc3aB045dDA812EEd76e",
		swapAddress: "0x969e50aeB7D4Fa170aF1ff5a5FD692ef5A75E189",
	},
};

// Initialize Admin instance from environment variables
export const adminContract = new Admin(
	process.env.RPC_URL || "http://localhost:9944",
	process.env.PRIVATE_KEY || "",
	process.env.STABLE_COIN_ADDRESS ||
		"0xd94548EcaDc804e4f30e9aee8ceB7ccD915855d3",
	process.env.TOKEN_FACTORY_ADDRESS ||
		"0xB500ebe89Eb51896B2C002e765d7516a9F44eD90",
	process.env.SWAP_ADDRESS || "0xD50474c7b28c7d7c2063489aE311b11a090867dC"
);

// Initialize User instance from environment variables
export const userContract = (private_key: string) =>
	new Admin(
		process.env.RPC_URL || "http://localhost:9944",
		private_key,
		process.env.STABLE_COIN_ADDRESS ||
			"0xd94548EcaDc804e4f30e9aee8ceB7ccD915855d3",
		process.env.TOKEN_FACTORY_ADDRESS ||
			"0xB500ebe89Eb51896B2C002e765d7516a9F44eD90",
		process.env.SWAP_ADDRESS || "0xD50474c7b28c7d7c2063489aE311b11a090867dC"
	);
