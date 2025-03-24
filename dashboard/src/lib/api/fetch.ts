import { err, ok } from "neverthrow";

interface AppError {
	status: number;
	message: string;
}

export async function post<T>(body: unknown) {
	try {
		const response = await fetch("", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				authorization: window.localStorage.getItem("access_token") ?? "",
			},
			body: JSON.stringify(body),
		});

		if (!response.ok) {
			throw new Error(`Response status: ${response.status}`);
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
			status,
			message,
		} as AppError);
	}
}

export async function get<T>() {
	try {
		const response = await fetch("", {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				authorization: window.localStorage.getItem("access_token") ?? "",
			},
		});

		if (!response.ok) {
			throw new Error(`Response status: ${response.status}`);
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
			status,
			message,
		} as AppError);
	}
}
