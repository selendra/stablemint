import { Request } from "express";
import User from "../models/user.model";
import { DecodedToken } from "../middleware/auth.middleware";

export async function isAdmin(userId: string) {
	const admin = await User.findOne({ _id: userId, role: "ADMIN" });
	if (!admin) {
		throw Error("Forbidden. Admin only.");
	}
	return admin;
}

export function getAuth(req: Request) {
	if (!("user" in req)) {
		throw Error("Unauthorized");
	}

	return req.user as DecodedToken;
}
