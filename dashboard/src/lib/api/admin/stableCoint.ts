import { adminPath, get, post } from "../fetch";
import {
	AccountWhiteListStatusResponse,
	BalanceResponse,
	TotalSupplyResponse,
	WhiteListResponse,
	WithdrawResponse,
} from "../types";

export async function getStableCoinBalance({ address }: { address: string }) {
	const request = await get<BalanceResponse>(
		adminPath(`stablecoin/balance/${address}`)
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getStableCoinTotalSupply() {
	const request = await get<TotalSupplyResponse>(
		adminPath(`stablecoin/total-supply`)
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function minStableCoin(body: {
	amount: number;
	toAddress: string;
}) {
	const request = await post<WithdrawResponse>(
		adminPath(`stablecoin/mint`),
		body
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function withdrawMoney(body: {
	amount: number;
	withdrawerAddress: string;
	reason: string;
}) {
	const request = await post<WithdrawResponse>(
		adminPath(`stablecoin/withdraw`),
		body
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function transferKhr(body: { amount: number; addresses: string }) {
	const request = await post<WithdrawResponse>(
		adminPath(`stablecoin/transfer`),
		body
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function addWhiteList(body: { address: string }) {
	const request = await post<WhiteListResponse>(
		adminPath(`/stablecoin/whitelist/add`),
		body
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function removeWhiteList(body: { address: string }) {
	const request = await post<WhiteListResponse>(
		adminPath(`/stablecoin/whitelist/remove`),
		body
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}

export async function getAccountWhitelistStatus({
	address,
}: {
	address: string;
}) {
	const request = await get<AccountWhiteListStatusResponse>(
		adminPath(`/stablecoin/whitelist/${address}`)
	);

	if (request.isErr()) {
		throw request.error;
	}

	return request.value;
}
