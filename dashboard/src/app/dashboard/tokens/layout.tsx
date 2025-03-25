"use client";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { usePathname, useRouter } from "next/navigation";
import React, { ReactNode } from "react";

export default function TokenLayout({ children }: { children: ReactNode }) {
	const router = useRouter();
	const pathname = usePathname();
	return (
		<Tabs value={pathname} onValueChange={(value) => router.push(value)}>
			<TabsList className="h-auto p-1 items-start">
				<TabsTrigger
					className="p-2 px-6 w-min cursor-pointer"
					value="/dashboard/tokens"
				>
					All
				</TabsTrigger>
				<TabsTrigger
					className="p-2 px-6 w-min cursor-pointer"
					value="/dashboard/tokens/created"
				>
					Created
				</TabsTrigger>
				<TabsTrigger
					className="p-2 px-6 w-min cursor-pointer"
					value="/dashboard/tokens/requested"
				>
					Requested
				</TabsTrigger>
				<TabsTrigger
					className="p-2 px-6 w-min cursor-pointer"
					value="/dashboard/tokens/rejected"
				>
					Rejected
				</TabsTrigger>
			</TabsList>
			{children}
		</Tabs>
	);
}
