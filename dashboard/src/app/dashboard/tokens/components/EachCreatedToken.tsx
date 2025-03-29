import { Button } from "@/components/ui/button";
import { TableCell, TableRow } from "@/components/ui/table";
import { Token } from "@/lib/api/token";
import React from "react";

export default function EachCreatedToken({ token }: { token: Token }) {
	return (
		<TableRow>
			<TableCell className="font-normal">{token?.name ?? "Unknown"}</TableCell>
			<TableCell className="font-normal">
				{token?.symbol ?? "Unknown"}
			</TableCell>
			<TableCell className="font-normal">{token.token_address}</TableCell>
			<TableCell className="font-normal">
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: "KHR",
				}).format(token.stable_coin_amount)}
			</TableCell>
			<TableCell className="font-normal">
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: "KHR",
				}).format(1.0)}{" "}
				/{" "}
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: token.symbol,
				}).format(token.ratio)}
			</TableCell>
			<TableCell className="font-normal">
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: token.symbol,
				}).format(token.ratio * token.stable_coin_amount)}
			</TableCell>
			<TableCell className="font-normal">{token.status}</TableCell>
			<TableCell>
				<div className="flex gap-2 place-content-end">
					<Button disabled>Approve</Button>
					<Button variant={"destructive"} disabled>
						Reject
					</Button>
				</div>
			</TableCell>
		</TableRow>
	);
}
