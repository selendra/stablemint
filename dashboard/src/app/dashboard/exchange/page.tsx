"use client";
import {
	Table,
	TableBody,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { Input } from "@/components/ui/input";
import React from "react";
import EachToken from "./components/EachToken";
import { useQuery } from "@tanstack/react-query";
import { getCreatedTokens } from "@/lib/api/token";

export default function Exchange() {
	const { data } = useQuery({
		queryKey: ["createdTokens"],
		queryFn: getCreatedTokens,
	});

	return (
		<div className="space-y-4">
			<div className="">
				<Input placeholder="Search" className="p-6" />
			</div>
			<div className="p-2 border rounded-sm">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead>Token</TableHead>
							<TableHead>Price</TableHead>
							<TableHead>Total Supply</TableHead>
							<TableHead>Market Cap</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{data?.map((token) => (
							<EachToken key={token._id} token={token} />
						))}
					</TableBody>
				</Table>
			</div>
		</div>
	);
}
