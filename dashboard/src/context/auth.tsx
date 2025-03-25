"use client";
import { getMe } from "@/lib/api/user";
import { useQuery } from "@tanstack/react-query";
import { usePathname, useRouter } from "next/navigation";

import React, { createContext, ReactNode, useEffect, useMemo } from "react";

export const AuthContext = createContext({
	isAuth: false,
});

export default function AuthProvider({ children }: { children: ReactNode }) {
	const pathname = usePathname();
	const router = useRouter();

	const { data, isLoading } = useQuery({
		queryKey: ["me"],
		queryFn: getMe,
		retry: false,
	});

	const isAuth = useMemo(() => {
		return Boolean(data);
	}, [data]);

	useEffect(() => {
		if (!isLoading && !isAuth) {
			console.log("pathname", pathname);

			if (pathname.startsWith("/dashboard")) {
				router.push("/login");
			}
		}
	}, [isLoading, isAuth, pathname, router]);

	if (isLoading) {
		return <div>Loading...</div>;
	}

	return (
		<AuthContext.Provider value={{ isAuth }}>{children}</AuthContext.Provider>
	);
}
