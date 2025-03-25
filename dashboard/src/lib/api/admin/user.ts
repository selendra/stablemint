import { adminPath, get } from "../fetch";
import { User } from "../types";

export async function getAllUsers() {
	const request = await get<User[]>(adminPath(`users`));
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
