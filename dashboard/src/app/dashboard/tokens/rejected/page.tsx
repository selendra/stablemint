"use client";

import {
	Table,
	TableBody,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import React from "react";
import EachRejectedToken from "./components/EachRejectedToken";
import { useQuery } from "@tanstack/react-query";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import { TabsContent } from "@/components/ui/tabs";

export default function RejectedTokens() {
	const { data } = useQuery({
		queryKey: ["rejectedTokens"],
		queryFn: getAllLoyaltyTokens,
	});
	return (
		<TabsContent value="/dashboard/tokens/rejected">
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
						<EachRejectedToken key={token} token={token} />
					))}
				</TableBody>
			</Table>
		</TabsContent>
	);
}
