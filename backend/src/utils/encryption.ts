// const crypto = require("crypto");

import crypto from "crypto";

class Encrypter {
	algorithm = "aes-192-cbc";
	key: ReturnType<typeof crypto.scryptSync>;

	constructor(encryptionKey: string) {
		this.key = crypto.scryptSync(encryptionKey, "salt", 24);
	}

	encrypt(clearText: string) {
		const iv = crypto.randomBytes(16);
		const cipher = crypto.createCipheriv(this.algorithm, this.key, iv);
		const encrypted = cipher.update(clearText, "utf8", "hex");
		return [
			encrypted + cipher.final("hex"),
			Buffer.from(iv).toString("hex"),
		].join("|");
	}

	dencrypt(encryptedText: string) {
		const [encrypted, iv] = encryptedText.split("|");
		if (!iv) throw new Error("IV not found");
		const decipher = crypto.createDecipheriv(
			this.algorithm,
			this.key,
			Buffer.from(iv, "hex")
		);
		return decipher.update(encrypted, "hex", "utf8") + decipher.final("utf8");
	}
}
