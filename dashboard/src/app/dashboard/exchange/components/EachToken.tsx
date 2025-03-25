import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { TableCell, TableRow } from "@/components/ui/table";

import { Token } from "@/lib/api/token";
import React from "react";

export default function EachToken({ token }: { token: Token }) {
	return (
		<TableRow>
			<TableCell className="">
				<div className="flex gap-2 place-items-center">
					<Avatar>
						<AvatarFallback>{token.symbol}</AvatarFallback>
					</Avatar>
					<div>
						<h2>{token.name}</h2>
						<p className="text-xs">{token.token_address}</p>
					</div>
				</div>
			</TableCell>
			<TableCell className="">
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: "KHR",
					maximumFractionDigits: 8,
				}).format(token.ratio / token.stable_coin_amount)}
			</TableCell>
			<TableCell className="">
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: token.symbol,
				}).format(token.stable_coin_amount * token.ratio)}
			</TableCell>
			<TableCell>
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: "KHR",
					maximumFractionDigits: 8,
				}).format(token.stable_coin_amount)}
			</TableCell>
		</TableRow>
	);
}
