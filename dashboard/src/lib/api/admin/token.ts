import { adminPath, get, post } from "../fetch";
import {
	AllTokensResponse,
	BalanceResponse,
	MintResponse,
	TokenInfo,
} from "../types";

export interface CreateTokenResponse {
	tokenAddress: string;
	success: boolean;
}

export async function createLoyaltyToken(body: {
	token_id: string;
	name: string;
	symbol: string;
	tokenOwner: string;
	tokensPerStableCoin: number;
}) {
	const request = await post<CreateTokenResponse>(
		adminPath(`/token/create`),
		body
	);

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
	const request = await get<BalanceResponse>(
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

export async function getLoyaltyTokenSupply({
	tokenAddress,
}: {
	tokenAddress: string;
}) {
	const request = await get<BalanceResponse>(
		adminPath(`/token/supply/${tokenAddress}`)
	);
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getLoyaltyTokenInfo({
	tokenAddress,
}: {
	tokenAddress: string;
}) {
	const request = await get<TokenInfo>(
		adminPath(`/token/info/${tokenAddress}`)
	);
	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
