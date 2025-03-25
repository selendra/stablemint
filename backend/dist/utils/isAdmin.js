"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.isAdmin = isAdmin;
exports.getAuth = getAuth;
const user_model_1 = __importDefault(require("../models/user.model"));
async function isAdmin(userId) {
    const admin = await user_model_1.default.findOne({ _id: userId, role: "ADMIN" });
    if (!admin) {
        throw Error("Forbidden. Admin only.");
    }
    return admin;
}
function getAuth(req) {
    if (!("user" in req)) {
        throw Error("Unauthorized");
    }
    return req.user;
}
