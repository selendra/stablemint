"use client";
import { useQuery } from "@tanstack/react-query";
import { getStableCoinTotalSupply } from "@/lib/api/admin/stableCoint";
import {
	Card,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { getAllLoyaltyTokens } from "@/lib/api/admin/token";

export default function Page() {
	const { data: totalSupply } = useQuery({
		queryKey: ["khrTotalSuppy"],
		queryFn: getStableCoinTotalSupply,
	});

	const { data: tokens } = useQuery({
		queryKey: ["tokens"],
		queryFn: getAllLoyaltyTokens,
	});

	return (
		<div className="grid grid-cols-3 gap-6">
			<Card>
				<CardHeader className="flex flex-col space-y-4 p-0">
					<div className="flex flex-1 flex-col justify-center gap-1 px-6">
						<CardTitle>Market Cap</CardTitle>
						<CardDescription>Total supply of KHR token</CardDescription>
					</div>
					<div className="flex">
						<button className="flex flex-1 flex-col justify-center gap-1 border-t px-6 text-left even:border-l data-[active=true]:bg-muted/50 sm:border-l sm:border-t-0 ">
							<span className="text-lg font-bold leading-none sm:text-3xl">
								{new Intl.NumberFormat("en-GB", {
									style: "currency",
									currency: "KHR",
								}).format(totalSupply?.totalSupply ?? 0)}
							</span>
						</button>
					</div>
				</CardHeader>
			</Card>
			<Card>
				<CardHeader className="flex flex-col space-y-4 p-0">
					<div className="flex flex-1 flex-col justify-center gap-1 px-6">
						<CardTitle>Loyalty Tokens</CardTitle>
						<CardDescription>
							Total supply of loyalty tokens base on KHR
						</CardDescription>
					</div>
					<div className="flex">
						<button className="flex flex-1 flex-col justify-center gap-1 border-t px-6 text-left even:border-l data-[active=true]:bg-muted/50 sm:border-l sm:border-t-0 ">
							<span className="text-lg font-bold leading-none sm:text-3xl">
								{new Intl.NumberFormat("en-GB", {
									style: "decimal",
									currency: "KHR",
								}).format(tokens?.tokens.length ?? 0)}
							</span>
						</button>
					</div>
				</CardHeader>
			</Card>

			<Card>
				<CardHeader className="flex flex-col space-y-4 p-0">
					<div className="flex flex-1 flex-col justify-center gap-1 px-6">
						<CardTitle>Market Cap</CardTitle>
						<CardDescription>Total supply of KHR token</CardDescription>
					</div>
					<div className="flex">
						<button className="flex flex-1 flex-col justify-center gap-1 border-t px-6 text-left even:border-l data-[active=true]:bg-muted/50 sm:border-l sm:border-t-0 ">
							<span className="text-lg font-bold leading-none sm:text-3xl">
								{new Intl.NumberFormat("en-GB", {
									style: "currency",
									currency: "KHR",
								}).format(totalSupply?.totalSupply ?? 0)}
							</span>
						</button>
					</div>
				</CardHeader>
			</Card>
		</div>
	);
}
