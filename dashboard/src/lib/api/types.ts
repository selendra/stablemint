export interface User {
	id: string;
	name: string;
	email: string;
	address?: string;
}

export interface LogInRespose {
	message: string;
	token: string;
	user: User;
}

export interface BalanceRespose {
	balance: number;
}

export interface AllTokensResponse {
	tokens: string[];
}

export interface MintResponse {
	success: boolean;
	error?: string;
	blockNumber?: string;
	transactionHash?: string;
}

export interface AccountWhiteListStatusResponse {
	isWhitelisted: boolean;
}

export interface WhiteListResponse {
	success: boolean;
	result: unknown;
}

export interface WithdrawResponse {
	success: boolean;
	result: unknown;
}

export interface TotalSupplyResponse {
	totalSupply: number;
}
