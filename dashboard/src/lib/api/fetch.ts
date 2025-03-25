import { err, ok } from "neverthrow";

const api = `${process.env.NEXT_PUBLIC_API}/api`;
export const authPath = (path: string) => `${api}/auth/${path}`;
export const adminPath = (path: string) => `${api}/contract/${path}`;

interface AppError {
	error: string;
	message: string;
}

export async function post<T>(path: string, body: unknown) {
	try {
		const response = await fetch(path, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				authorization: window.localStorage.getItem("access_token") ?? "",
			},
			body: JSON.stringify(body),
		});

		if (!response.ok) {
			const error = (await response.json()) as AppError;
			throw error;
		}

		return ok((await response.json()) as T);
	} catch (e) {
		console.log("fucking e", e);

		const error = e as Error;
		const status =
			typeof e === "object" && e !== null && "status" in e
				? // eslint-disable-next-line @typescript-eslint/no-explicit-any
				  (e as any).status
				: 0;
		const message = error.message || "Unknown error";

		return err({
			error: status,
			message,
		} as AppError);
	}
}

export async function get<T>(path: string) {
	try {
		const response = await fetch(path, {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				authorization: window.localStorage.getItem("access_token") ?? "",
			},
		});

		if (!response.ok) {
			const error = (await response.json()) as AppError;
			throw error;
		}

		return ok((await response.json()) as T);
	} catch (e) {
		const error = e as Error;
		const status =
			typeof e === "object" && e !== null && "status" in e
				? // eslint-disable-next-line @typescript-eslint/no-explicit-any
				  (e as any).status
				: 0;
		const message = error.message || "Unknown error";

		return err({
			error: status,
			message,
		} as AppError);
	}
}
