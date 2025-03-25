"use client";

import React from "react";
import {
	Table,
	TableBody,
	TableCaption,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useQuery } from "@tanstack/react-query";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import EachToken from "./components/EachToken";

export default function Tokens() {
	const { data } = useQuery({
		queryKey: ["tokens"],
		queryFn: getAllLoyaltyTokens,
	});

	return (
		<Table>
			<TableCaption>A list of your recent invoices.</TableCaption>
			<TableHeader>
				<TableRow>
					<TableHead className="w-[100px]">Address</TableHead>
					<TableHead>Status</TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{data?.tokens.map((token) => (
					<EachToken key={token} token={token} />
				))}
			</TableBody>
		</Table>
	);
}
