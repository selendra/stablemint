"use client";

import React from "react";
import {
	Table,
	TableBody,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useQuery } from "@tanstack/react-query";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import EachToken from "./components/EachToken";
import { TabsContent } from "@/components/ui/tabs";

export default function Tokens() {
	const { data } = useQuery({
		queryKey: ["tokens"],
		queryFn: getAllLoyaltyTokens,
	});

	return (
		<TabsContent value="/dashboard/tokens">
			<Table>
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
						<EachToken key={token} token={token} />
					))}
				</TableBody>
			</Table>
		</TabsContent>
	);
}
