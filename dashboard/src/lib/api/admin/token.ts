import { adminPath, get, post } from "../fetch";
import { AllTokensResponse, BalanceRespose, MintResponse } from "../types";

export async function createLoyaltyToken(body: {
	name: string;
	symbol: string;
	tokenOwner: number;
	tokensPerStableCoin: number;
}) {
	const request = await post<MintResponse>(adminPath(`/token/create`), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function mintLoyaltyToken(body: {
	tokenAddress: string;
	toAddress: string;
	amount: number;
}) {
	const request = await post<MintResponse>(adminPath(`/token/mint`), body);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getLoyaltyTokenBalance({
	tokenAddress,
	accountAddress,
}: {
	tokenAddress: string;
	accountAddress: string;
}) {
	const request = await get<BalanceRespose>(
		adminPath(`/token/balance/${tokenAddress}/${accountAddress}`)
	);
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getAllLoyaltyTokens() {
	const request = await get<AllTokensResponse>(adminPath(`/token/all`));
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
