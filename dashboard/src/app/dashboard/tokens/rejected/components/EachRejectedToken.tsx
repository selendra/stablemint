import { TableCell, TableRow } from "@/components/ui/table";
import {
	getLoyaltyTokenInfo,
	getLoyaltyTokenSupply,
} from "@/lib/api/admin/token";
import { useQuery } from "@tanstack/react-query";
import React from "react";

export default function EachRejectedToken({ token }: { token: string }) {
	const { data: supply } = useQuery({
		queryKey: ["totalSpply", token],
		queryFn: () => getLoyaltyTokenSupply({ tokenAddress: token }),
	});

	const { data: info } = useQuery({
		queryKey: ["tokenInfo", token],
		queryFn: () => getLoyaltyTokenInfo({ tokenAddress: token }),
	});

	return (
		<TableRow>
			<TableCell className="font-mono font-normal">
				{info?.name ?? "Unknown"}
			</TableCell>
			<TableCell className="font-mono font-normal">
				{info?.symbol ?? "Unknown"}
			</TableCell>
			<TableCell className="font-mono font-normal">{token}</TableCell>
			<TableCell>{supply?.balance ?? 0}</TableCell>
		</TableRow>
	);
}
