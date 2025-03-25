import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { getLoyaltyTokenBalance } from "@/lib/api/admin/stableCoint";
import { Token } from "@/lib/api/token";
import { cn } from "@/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { CheckIcon, TriangleAlert } from "lucide-react";
import React from "react";

export default function EachTokenHome({
	token,
	userAddress,
}: {
	token: Token;
	userAddress: string;
}) {
	const { data: tokenBalance } = useQuery({
		queryKey: ["user", userAddress, "token", token._id],
		queryFn: () =>
			getLoyaltyTokenBalance({
				tokenAddress: token.token_address ?? "",
				accountAddress: userAddress,
			}),
	});

	return (
		<div
			className="flex gap-2 place-items-center border rounded-sm p-2"
			key={token._id}
		>
			<Avatar className="bg-accent">
				<AvatarFallback className="text-xs">{token.symbol}</AvatarFallback>
			</Avatar>
			<div className="flex-grow">
				<div className="flex gap-2 place-items-center">
					<h2>{token.name}</h2>
					<Button
						size={"icon"}
						className={cn("rounded-full w-5 h-5 p-1", {
							"bg-amber-300": token.status === "PENDING",
							"bg-green-500": token.status === "CREATED",
							"bg-red-500": token.status === "REJECTED",
						})}
						variant={"outline"}
					>
						{token.status === "CREATED" && <CheckIcon />}
						{token.status === "PENDING" && <TriangleAlert />}
						{token.status === "REJECTED" && <TriangleAlert />}
					</Button>
				</div>
				<p className="text-xs">{token.token_address}</p>
			</div>

			{new Intl.NumberFormat("en-GB", {
				style: "decimal",
			}).format(tokenBalance?.balance ?? 0)}
		</div>
	);
}
