"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = require("express");
const loyaltyController = __importStar(require("../controllers/loyalty.controller"));
const auth_middleware_1 = require("../middleware/auth.middleware");
const express_validator_1 = require("express-validator");
const router = (0, express_1.Router)();
router.get("/", auth_middleware_1.authMiddleware, loyaltyController.getAllTokens);
router.get("/created", auth_middleware_1.authMiddleware, loyaltyController.getAllCreatedTokens);
router.get("/pending", auth_middleware_1.authMiddleware, loyaltyController.getAllPendingTokens);
router.get("/rejected", auth_middleware_1.authMiddleware, loyaltyController.getAllRejectedTokens);
router.post("/request", [
    (0, express_validator_1.body)("name").isString().withMessage("name is required"),
    (0, express_validator_1.body)("symbol").isString().withMessage("symbol is required"),
    (0, express_validator_1.body)("stable_coin_amount")
        .isNumeric()
        .withMessage("stable_coin_amount is required"),
    (0, express_validator_1.body)("ratio").isNumeric().withMessage("ratio is required"),
    auth_middleware_1.authMiddleware,
], loyaltyController.requestToken);
router.post("/reject", [
    (0, express_validator_1.body)("token_id").isString().withMessage("token_id is required"),
    auth_middleware_1.authMiddleware,
], loyaltyController.rejectToken);
exports.default = router;
