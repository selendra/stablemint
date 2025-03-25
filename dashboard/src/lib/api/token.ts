import { get, post, tokenPath } from "./fetch";

export interface Token {
	_id: string;
	name: string;
	symbol: string;
	token_address: string | null;
	owner_id: string;
	status: "PENDING" | "CREATED" | "REJECTED";
	rejected_reason: string | null;
	stable_coin_amount: number;
	ratio: number;
	createdAt: string;
	updatedAt: string;
}

export async function getAllTokens() {
	const request = await get<Token[]>(tokenPath(""));

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getTokensByOwnerId({ userId }: { userId: string }) {
	const request = await get<Token[]>(tokenPath(`created-by/${userId}`));

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getPendingTokens() {
	const request = await get<Token[]>(tokenPath("pending"));

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getRejectedTokens() {
	const request = await get<Token[]>(tokenPath("rejected"));

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getCreatedTokens() {
	const request = await get<Token[]>(tokenPath("created"));

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function rejectToken(body: { token_id: string }) {
	const request = await post<Token>(tokenPath("reject"), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function requestCreateToken(body: {
	name: string;
	symbol: string;
	stable_coin_amount: number;
	ratio: number;
}) {
	const request = await post<Token>(tokenPath("request"), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
