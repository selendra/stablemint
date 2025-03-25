import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { TableCell, TableRow } from "@/components/ui/table";
import {
	addWhiteList,
	getAccountWhitelistStatus,
	getStableCoinBalance,
	removeWhiteList,
} from "@/lib/api/admin/stableCoint";
import { releaseAirDrop } from "@/lib/api/admin/user";
import { User } from "@/lib/api/types";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React, { useState } from "react";

export default function EachUser({ user }: { user: User }) {
	const [whitelisting, setWhitelisting] = useState(false);
	const { data: balance } = useQuery({
		queryKey: ["stableCoinBalance", user._id],
		queryFn: () => {
			return getStableCoinBalance({ address: user.address! });
		},
	});

	const { data: accountStatus, refetch } = useQuery({
		queryKey: ["accountWhiteList", user._id],
		queryFn: () => {
			return getAccountWhitelistStatus({ address: user.address! });
		},
	});

	const airdrop = useMutation({
		mutationFn: releaseAirDrop,
	});

	const addWhitelist = useMutation({
		mutationKey: ["addWhiteList", user._id],
		mutationFn: addWhiteList,
	});

	const removeWhitelist = useMutation({
		mutationKey: ["removeWhiteList", user._id],
		mutationFn: removeWhiteList,
	});

	async function handleWhitelist() {
		try {
			setWhitelisting(true);
			await addWhitelist.mutateAsync({ address: user.address! });
			await airdrop.mutateAsync({ toAddress: user.address!, amount: 1 });
			await refetch();
			setWhitelisting(false);
		} catch (error) {
			console.log(error);
			setWhitelisting(false);
		}
	}

	return (
		<TableRow>
			<TableCell>{user.name}</TableCell>
			<TableCell>{user.email}</TableCell>
			<TableCell>{user.address}</TableCell>
			<TableCell>
				{new Intl.NumberFormat("en-GB", {
					style: "currency",
					currency: "KHR",
					currencyDisplay: "code",
				}).format(balance?.balance ?? 0)}
			</TableCell>
			<TableCell>
				<Label>
					<Checkbox checked={accountStatus?.isWhitelisted} />
					{accountStatus?.isWhitelisted ? "Whitelisted" : "Unwhitelisted"}
				</Label>
			</TableCell>

			<TableCell>
				{!Boolean(accountStatus?.isWhitelisted) && (
					<Button disabled={whitelisting} onClick={handleWhitelist}>
						{whitelisting && <Loader2 className="animate-spin" />}
						Activate
					</Button>
				)}

				{Boolean(accountStatus?.isWhitelisted) && (
					<Button
						disabled={addWhitelist.isPending || removeWhitelist.isPending}
						onClick={async () => {
							await removeWhitelist.mutateAsync({ address: user.address! });
							await refetch();
						}}
					>
						{removeWhitelist.isPending && <Loader2 className="animate-spin" />}
						Decativate
					</Button>
				)}
			</TableCell>
		</TableRow>
	);
}
