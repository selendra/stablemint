import dotenv from "dotenv";
import { Admin } from "../utils/contract";

dotenv.config();

export default {
  port: process.env.PORT || 5000,
  mongodbUri: process.env.MONGODB_URI || "mongodb://localhost:27017/stablemint",
  jwtSecret: process.env.JWT_SECRET || "default_jwt_secret",
  jwtExpiresIn: process.env.JWT_EXPIRES_IN || "7d",
  contract: {
    stableCoinAdress:
      process.env.STABLE_COIN_ADDRESS ||
      "0xffFEdB07dbc5A93A3c7653930e46Bd9332468559",
    tokenFactoryAddress:
      process.env.TOKEN_FACTORY_ADDRESS ||
      "0x376e34036b77704B7558Dc3aB045dDA812EEd76e",
    swapAddress:
      process.env.SWAP_ADDRESS || "0x969e50aeB7D4Fa170aF1ff5a5FD692ef5A75E189",
  },
};

// Initialize Admin instance from environment variables
export const adminContract = new Admin(
  process.env.RPC_URL || "http://localhost:9944",
  process.env.PRIVATE_KEY || "",
  process.env.STABLE_COIN_ADDRESS ||
    "0xffFEdB07dbc5A93A3c7653930e46Bd9332468559",
  process.env.TOKEN_FACTORY_ADDRESS ||
    "0x376e34036b77704B7558Dc3aB045dDA812EEd76e",
  process.env.SWAP_ADDRESS || "0x969e50aeB7D4Fa170aF1ff5a5FD692ef5A75E189"
);
