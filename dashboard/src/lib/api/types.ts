export interface User {
	id: string;
	name: string;
	email: string;
}
export interface LogInRespose {
	message: string;
	token: string;
	user: User;
}
