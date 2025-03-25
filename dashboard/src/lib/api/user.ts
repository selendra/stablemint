import { authPath, get, post } from "./fetch";
import { LogInRespose, User } from "./types";

export async function getMe() {
	const user = await get<User>(authPath("/me"));

	if (user.isErr()) {
		throw user.error;
	}

	return user.value;
}

export async function userLogin({
	email,
	password,
}: {
	email: string;
	password: string;
}) {
	const login = await post<LogInRespose>(authPath("/login"), {
		email,
		password,
	});

	if (login.isErr()) {
		throw login.error;
	}

	return login.value;
}

export async function userRegister({
	name,
	email,
	password,
}: {
	name: string;
	email: string;
	password: string;
}) {
	const register = await post<LogInRespose>(authPath("/register"), {
		name,
		email,
		password,
	});

	if (register.isErr()) {
		throw register.error;
	}

	return register.value;
}
