import { Button } from "@/components/ui/button";
import { TableCell, TableRow } from "@/components/ui/table";
import { transferKhr } from "@/lib/api/admin/stableCoint";
import { createLoyaltyToken, mintLoyaltyToken } from "@/lib/api/admin/token";
import { rejectToken, Token } from "@/lib/api/token";
import { getUserById } from "@/lib/api/user";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React, { useState } from "react";

export default function EachRequestedToken({ token }: { token: Token }) {
	const [approving, setApproving] = useState(false);
	const reject = useMutation({
		mutationFn: rejectToken,
	});

	const { data: user } = useQuery({
		queryKey: ["user", token.owner_id],
		queryFn: () => getUserById({ userId: token.owner_id }),
	});

	const createToken = useMutation({
		mutationFn: createLoyaltyToken,
	});

	const transferStableCoin = useMutation({
		mutationFn: transferKhr,
	});

	const mintToken = useMutation({
		mutationFn: mintLoyaltyToken,
	});

	async function handleApprove() {
		try {
			setApproving(true);
			const created = await createToken.mutateAsync({
				token_id: token._id,
				name: token.name,
				symbol: token.symbol,
				tokenOwner: user!.address!,
				tokensPerStableCoin: token.ratio,
			});

			await transferStableCoin.mutateAsync({
				addresses: created.tokenAddress,
				amount: token.stable_coin_amount,
			});

			await mintToken.mutateAsync({
				amount: token.stable_coin_amount * token.ratio,
				toAddress: user!.address!,
				tokenAddress: created.tokenAddress,
			});
			setApproving(false);
		} catch (error) {
			console.log(error);

			setApproving(false);
		}
	}
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
					<Button disabled={approving} onClick={handleApprove}>
						{approving && <Loader2 className="animate-spin" />}
						Approve
					</Button>
					<Button
						variant={"destructive"}
						onClick={async () => {
							await reject.mutateAsync({
								token_id: token._id,
							});
						}}
						disabled={reject.isPending}
					>
						{reject.isPending && <Loader2 className="animate-spin" />}
						Reject
					</Button>
				</div>
			</TableCell>
		</TableRow>
	);
}
