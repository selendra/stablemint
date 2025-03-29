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
import { getRejectedTokens } from "@/lib/api/token";

export default function RejectedTokens() {
	const { data } = useQuery({
		queryKey: ["rejectedTokens"],
		queryFn: getRejectedTokens,
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
					<EachRejectedToken key={token._id} token={token} />
				))}
			</TableBody>
		</Table>
	);
}
