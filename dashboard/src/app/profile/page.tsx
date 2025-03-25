"use client";

import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { getStableCoinBalance, transferKhr } from "@/lib/api/admin/stableCoint";
import { requestCreateToken } from "@/lib/api/token";
import { getMe } from "@/lib/api/user";
import { useMutation, useQuery } from "@tanstack/react-query";
import React, { useState } from "react";

export default function Profile() {
	const { data: me } = useQuery({
		queryKey: ["me"],
		queryFn: getMe,
	});

	const { data: khr } = useQuery({
		queryKey: ["myKhr"],
		queryFn: () => getStableCoinBalance({ address: me?.address ?? "" }),
		enabled: Boolean(me?.address),
	});

	const [state, setState] = useState({
		name: "",
		symbol: "",
		stable_coin_amount: 0,
		ratio: 0,
	});

	function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
		const { name, value } = e.target;
		setState({ ...state, [name]: value });
	}

	const request = useMutation({
		mutationFn: requestCreateToken,
	});

	const transferStableCoin = useMutation({
		mutationFn: transferKhr,
	});

	async function handleCreate() {
		try {
			await transferStableCoin.mutateAsync({
				addresses: "0x8cfc1EeCA441a4554Fc3DFcea1fcBf25749C4ecD",
				amount: state.stable_coin_amount,
			});
			await request.mutateAsync(state);
		} catch (error) {
			console.log(error);
		}
	}

	return (
		<div className="p-4 space-y-4">
			<div className="bg-accent w-full aspect-video rounded-sm p-4">
				<h1>{me?.name}</h1>
				<p>{me?.address}</p>
				<p>
					{new Intl.NumberFormat("en-GB", {
						style: "currency",
						currency: "KHR",
					}).format(khr?.balance ?? 0)}
				</p>
			</div>
			<div>
				<Dialog>
					<DialogTrigger asChild>
						<Button>Create Loyalty Point</Button>
					</DialogTrigger>
					<DialogContent>
						<DialogHeader>
							<DialogTitle>Are you absolutely sure?</DialogTitle>
							<DialogDescription>
								This action cannot be undone. This will permanently delete your
								account and remove your data from our servers.
							</DialogDescription>
							<Input
								value={state.name}
								name="name"
								placeholder="Name"
								onChange={handleChange}
							/>
							<Input
								value={state.symbol}
								name="symbol"
								placeholder="Symbol"
								onChange={handleChange}
							/>
							<Input
								value={state.stable_coin_amount}
								name="stable_coin_amount"
								placeholder="KHR Collateral"
								onChange={handleChange}
							/>
							<Input
								value={state.ratio}
								name="ratio"
								placeholder="Conversion ratio"
								onChange={handleChange}
							/>
						</DialogHeader>
						<DialogFooter>
							<Button onClick={handleCreate}>Create</Button>
						</DialogFooter>
					</DialogContent>
				</Dialog>
			</div>
		</div>
	);
}
