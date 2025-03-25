"use client";
import {
	TableHeader,
	TableRow,
	TableHead,
	TableBody,
	Table,
} from "@/components/ui/table";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import { useQuery } from "@tanstack/react-query";

import React from "react";
import EachRejectedToken from "../rejected/components/EachRejectedToken";
import { TabsContent } from "@/components/ui/tabs";

export default function CreatedTokens() {
	const { data } = useQuery({
		queryKey: ["createdTokens"],
		queryFn: getAllLoyaltyTokens,
	});
	return (
		<TabsContent value="/dashboard/tokens/created">
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
