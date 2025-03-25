"use strict";
// const crypto = require("crypto");
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const crypto_1 = __importDefault(require("crypto"));
class Encrypter {
    algorithm = "aes-192-cbc";
    key;
    constructor(encryptionKey) {
        this.key = crypto_1.default.scryptSync(encryptionKey, "salt", 24);
    }
    encrypt(clearText) {
        const iv = crypto_1.default.randomBytes(16);
        const cipher = crypto_1.default.createCipheriv(this.algorithm, this.key, iv);
        const encrypted = cipher.update(clearText, "utf8", "hex");
        return [
            encrypted + cipher.final("hex"),
            Buffer.from(iv).toString("hex"),
        ].join("|");
    }
    dencrypt(encryptedText) {
        const [encrypted, iv] = encryptedText.split("|");
        if (!iv)
            throw new Error("IV not found");
        const decipher = crypto_1.default.createDecipheriv(this.algorithm, this.key, Buffer.from(iv, "hex"));
        return decipher.update(encrypted, "hex", "utf8") + decipher.final("utf8");
    }
}
