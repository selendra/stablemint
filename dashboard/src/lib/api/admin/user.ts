import { adminPath, get, post } from "../fetch";
import { BalanceResponse, User } from "../types";

export async function getAllUsers() {
	const request = await get<User[]>(adminPath(`users`));
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getSel({ address }: { address: string }) {
	const request = await get<BalanceResponse>(adminPath(`users/sel/${address}`));
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function releaseAirDrop(body: {
	toAddress: string;
	amount: number;
}) {
	const request = await post<unknown>(adminPath("users/airdrop"), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function navtiveTransfer(body: {
	toAddress: string;
	amount: number;
}) {
	const request = await post<unknown>(adminPath("users/transfer"), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function tokenTransfer(body: {
	tokenAddress: string;
	toAddress: string;
	amount: number;
}) {
	const request = await post<unknown>(adminPath("token/transfer"), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
