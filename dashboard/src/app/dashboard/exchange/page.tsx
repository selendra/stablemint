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
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";
import { useQuery } from "@tanstack/react-query";

export default function Exchange() {
	const { data } = useQuery({
		queryKey: ["tokens"],
		queryFn: getAllLoyaltyTokens,
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
							<TableHead>Volume</TableHead>
							<TableHead>Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{data?.tokens.map((token) => (
							<EachToken key={token} token={token} />
						))}
					</TableBody>
				</Table>
			</div>
		</div>
	);
}
