"use client";
import {
	TableHeader,
	TableRow,
	TableHead,
	TableBody,
	Table,
} from "@/components/ui/table";
import { useQuery } from "@tanstack/react-query";

import React from "react";
import { getCreatedTokens } from "@/lib/api/token";
import EachCreatedToken from "./components/EachCreatedToken";

export default function CreatedTokens() {
	const { data } = useQuery({
		queryKey: ["createdTokens"],
		queryFn: getCreatedTokens,
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
					<EachCreatedToken key={token._id} token={token} />
				))}
			</TableBody>
		</Table>
	);
}
