"use client";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import Link from "next/link";
import { useMutation } from "@tanstack/react-query";
import { userRegister } from "@/lib/api/user";
import { useEffect, useState } from "react";

export function RegisterForm({
	className,
	...props
}: React.ComponentProps<"div">) {
	const [name, setName] = useState("");
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");

	const register = useMutation({
		mutationFn: userRegister,
	});

	useEffect(() => {
		if (register.data?.token) {
			window.localStorage.setItem("access_token", register.data.token);
			window.location.replace("/dashboard");
		}
	}, [register]);

	return (
		<div className={cn("flex flex-col gap-6", className)} {...props}>
			<Card>
				<CardHeader>
					<CardTitle>Register a new account</CardTitle>
					<CardDescription>
						Enter your real name, email, and password below to create new
						account
					</CardDescription>
				</CardHeader>
				<CardContent>
					<form
						onSubmit={async (e) => {
							e.preventDefault();
							await register.mutateAsync({
								email,
								password,
								name,
							});
						}}
					>
						<div className="flex flex-col gap-6">
							<div className="grid gap-3">
								<Label htmlFor="email">Full name</Label>
								<Input
									id="name"
									placeholder="Jonh Doe"
									required
									value={name}
									onChange={(e) => setName(e.target.value)}
								/>
							</div>
							<div className="grid gap-3">
								<Label htmlFor="email">Email</Label>
								<Input
									id="email"
									type="email"
									placeholder="m@example.com"
									required
									value={email}
									onChange={(e) => setEmail(e.target.value)}
								/>
							</div>
							<div className="grid gap-3">
								<div className="flex items-center">
									<Label htmlFor="password">Password</Label>
								</div>
								<Input
									id="password"
									type="password"
									required
									value={password}
									onChange={(e) => setPassword(e.target.value)}
								/>
							</div>
							<div className="flex flex-col gap-3">
								<Button
									type="submit"
									className="w-full"
									disabled={
										!name ||
										!password ||
										!email ||
										register.status === "pending"
									}
								>
									Register
								</Button>
							</div>
						</div>
						{register.error && (
							<p className="text-destructive">{register.error.message}</p>
						)}
						<div className="mt-4 text-center text-sm">
							Already have an account?{" "}
							<Link href="/login" className="underline underline-offset-4">
								Login
							</Link>
						</div>
					</form>
				</CardContent>
			</Card>
		</div>
	);
}
