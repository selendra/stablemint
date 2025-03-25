"use client";

import {
	Table,
	TableBody,
	TableCaption,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import { useQuery } from "@tanstack/react-query";
import React from "react";
import EachRequestedToken from "./components/EachRequestedToken";
import { TabsContent } from "@/components/ui/tabs";

export default function RequestedToken() {
	const { data } = useQuery({
		queryKey: ["requestedTokens"],
		queryFn: getAllLoyaltyTokens,
	});
	return (
		<TabsContent value="/dashboard/tokens/requested">
			<Table>
				<TableCaption>A list of your recent invoices.</TableCaption>
				<TableHeader>
					<TableRow>
						<TableHead>Name</TableHead>
						<TableHead>Symbol</TableHead>
						<TableHead>Address</TableHead>
						<TableHead>Supply</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{data?.tokens.map((token) => (
						<EachRequestedToken key={token} token={token} />
					))}
				</TableBody>
			</Table>
		</TabsContent>
	);
}
