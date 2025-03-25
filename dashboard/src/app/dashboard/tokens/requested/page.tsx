"use client";

import {
	Table,
	TableBody,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useQuery } from "@tanstack/react-query";
import React from "react";
import EachRequestedToken from "./components/EachRequestedToken";
import { getPendingTokens } from "@/lib/api/token";

export default function RequestedToken() {
	const { data } = useQuery({
		queryKey: ["requestedTokens"],
		queryFn: getPendingTokens,
	});
	return (
		<Table>
			<TableHeader>
				<TableRow>
					<TableHead>Name</TableHead>
					<TableHead>Symbol</TableHead>
					<TableHead>Address</TableHead>
					<TableHead>KHR Collateral</TableHead>
					<TableHead>Pegged Ratio</TableHead>
					<TableHead>Total Supply</TableHead>
					<TableHead>Status</TableHead>
					<TableHead></TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{data?.map((token) => (
					<EachRequestedToken key={token._id} token={token} />
				))}
			</TableBody>
		</Table>
	);
}
