import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { TableCell, TableRow } from "@/components/ui/table";
import {
	getLoyaltyTokenInfo,
	getLoyaltyTokenSupply,
} from "@/lib/api/admin/token";
import { useQuery } from "@tanstack/react-query";
import React from "react";

export default function EachToken({ token }: { token: string }) {
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
				<div className="flex gap-2 place-items-center">
					<Avatar>
						<AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
						<AvatarFallback>CN</AvatarFallback>
					</Avatar>
					<div>
						<h2>{info?.name ?? "Unknown"}</h2>
						<p>{info?.symbol ?? "Unknown"}</p>
					</div>
				</div>
			</TableCell>
			<TableCell className="font-mono font-normal">
				{supply?.balance ?? 0}
			</TableCell>
			<TableCell className="font-mono font-normal"></TableCell>
			<TableCell></TableCell>
		</TableRow>
	);
}
