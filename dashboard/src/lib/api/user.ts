import { get, post } from "./fetch";
import { LogInRespose, User } from "./types";

export async function getMe() {
	return await get<User>();
}

export async function login({
	email,
	password,
}: {
	email: string;
	password: string;
}) {
	return await post<LogInRespose>({ email, password });
}

export async function register({
	name,
	email,
	password,
}: {
	name: string;
	email: string;
	password: string;
}) {
	return await post<LogInRespose>({ name, email, password });
}
