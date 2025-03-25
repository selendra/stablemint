"use client";

import { Button } from "@/components/ui/button";
import {
	Card,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { getStableCoinBalance, transferKhr } from "@/lib/api/admin/stableCoint";
import { getTokensByOwnerId, requestCreateToken } from "@/lib/api/token";
import { getMe } from "@/lib/api/user";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Loader2, QrCodeIcon, ScanQrCodeIcon } from "lucide-react";
import Link from "next/link";
import React, { useState } from "react";
import QRCode from "react-qr-code";
import { Scanner } from "@yudiel/react-qr-scanner";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { getSel, navtiveTransfer, tokenTransfer } from "@/lib/api/admin/user";
import EachTokenHome from "./components/EachToken";

export default function HomePage() {
	const [open, setOpen] = useState(false);
	const [openQr, setOpenQr] = useState(false);
	const [openTransfer, setOpenTransfer] = useState(false);
	const [transfering, setTransfering] = useState(false);
	const [receiver, setReceiver] = useState("");
	const [amount, setAmount] = useState(0);
	const [tokenAddress, setTokenAddress] = useState("");
	const [openScanner, setOpenScanner] = useState(false);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState("");
	const [transferError, setTranferError] = useState("");
	const { data: me } = useQuery({
		queryKey: ["me"],
		queryFn: getMe,
	});

	const { data: khr, refetch: refetchKhr } = useQuery({
		queryKey: ["myKhr"],
		queryFn: () => getStableCoinBalance({ address: me?.address ?? "" }),
		enabled: Boolean(me?.address),
	});

	const { data: selBalance, refetch: refetchSel } = useQuery({
		queryKey: ["mySel", me?._id],
		queryFn: () => getSel({ address: me?.address ?? "" }),
		enabled: Boolean(me?.address),
	});

	const { data: createdTokens, refetch: refetchTokens } = useQuery({
		queryKey: ["tokensCreatedBy", me?._id],
		queryFn: () => getTokensByOwnerId({ userId: me?._id ?? "" }),
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

	const transferPoint = useMutation({
		mutationFn: tokenTransfer,
	});

	const transferSEL = useMutation({
		mutationFn: navtiveTransfer,
	});

	async function handleCreate() {
		try {
			setLoading(true);
			await transferStableCoin.mutateAsync({
				addresses: "0x8cfc1EeCA441a4554Fc3DFcea1fcBf25749C4ecD",
				amount: state.stable_coin_amount,
			});
			await request.mutateAsync(state);
			setOpen(false);
			setLoading(false);
		} catch (e) {
			console.log(e);
			const err = e as Error;
			setError(err?.message ?? "Something went wrong");
			setLoading(false);
		}
	}

	async function handleTransfer() {
		try {
			setTransfering(true);

			if (tokenAddress === "SEL") {
				await transferSEL.mutateAsync({
					toAddress: receiver,
					amount: amount,
				});
				await refetchSel();
			}

			if (tokenAddress === "KHR") {
				await transferStableCoin.mutateAsync({
					addresses: receiver,
					amount: amount,
				});
				await refetchKhr();
			}

			if (tokenAddress !== "KHR" && tokenAddress.startsWith("0x")) {
				await transferPoint.mutateAsync({
					amount,
					toAddress: receiver,
					tokenAddress,
				});
				await refetchTokens();
			}

			setTransfering(false);
			setOpenTransfer(false);
		} catch (error) {
			// console.log(error);
			const err = error as Error;
			setTranferError(err.message ?? "Something went wrong while transfering");
			setTransfering(false);
		}
	}

	return (
		<div className="p-4 space-y-4">
			<Card>
				<CardHeader className="flex flex-col space-y-4 px-4">
					<div className="flex w-full">
						<div className="flex flex-col justify-center gap-1 flex-grow">
							<CardTitle>{me?.name}</CardTitle>
							<CardDescription>{me?.address}</CardDescription>
						</div>
						<div className="space-x-2">
							<Button variant={"outline"} onClick={() => setOpenQr(true)}>
								<QrCodeIcon />
							</Button>
							<Button variant={"outline"} onClick={() => setOpenScanner(true)}>
								<ScanQrCodeIcon />
							</Button>
						</div>
					</div>
					<Separator />
					<div className="w-full flex place-content-between">
						<button className="flex flex-1 flex-col justify-center gap-1  text-left  ">
							<span className="text-lg font-bold leading-none sm:text-3xl">
								{new Intl.NumberFormat("en-GB", {
									style: "currency",
									currency: "KHR",
									minimumFractionDigits: 2,
								}).format(khr?.balance ?? 0)}
							</span>
						</button>
						<Separator orientation="vertical" className="h-10" />
						<button className="flex flex-1 flex-col justify-center gap-1  text-left  ">
							<span className="text-lg font-bold leading-none sm:text-3xl">
								{new Intl.NumberFormat("en-GB", {
									style: "currency",
									currency: "SEL",
									minimumFractionDigits: 6,
								}).format(selBalance?.balance ?? 0)}
							</span>
						</button>

						<Button variant={"outline"} asChild>
							<Link href="/dashboard">Dashboard</Link>
						</Button>
					</div>
				</CardHeader>
			</Card>

			<div className="flex place-content-between place-items-center">
				<h2 className="font-bold text-lg">Created Tokens</h2>
				<Button
					onClick={() => {
						setOpen(true);
					}}
					variant={"outline"}
				>
					Mint Point
				</Button>
			</div>
			<div className="space-y-2">
				{createdTokens?.map((token) => (
					<EachTokenHome
						key={token._id}
						token={token}
						userAddress={me?.address ?? ""}
					/>
				))}
			</div>
			<div>
				<Dialog open={open} onOpenChange={setOpen}>
					<DialogContent>
						<DialogHeader>
							<DialogTitle>Create Loyalty Point</DialogTitle>
						</DialogHeader>
						<Label>Token name</Label>
						<Input
							value={state.name}
							name="name"
							placeholder="Name"
							onChange={handleChange}
						/>
						<Label>Token symbol</Label>
						<Input
							value={state.symbol}
							name="symbol"
							placeholder="Symbol"
							onChange={handleChange}
						/>
						<Label>KHR Collateral</Label>
						<Input
							value={state.stable_coin_amount}
							name="stable_coin_amount"
							placeholder="KHR Collateral"
							onChange={handleChange}
						/>
						<Label>Conversion ratio</Label>
						<Input
							value={state.ratio}
							name="ratio"
							placeholder="Conversion ratio"
							onChange={handleChange}
						/>
						<DialogFooter>
							{!!error && <p className="text-destructive">{error}</p>}
							<Button className="w-full" onClick={handleCreate}>
								{loading && <Loader2 className="animate-spin" />}
								Create
							</Button>
						</DialogFooter>
					</DialogContent>
				</Dialog>

				<Dialog open={openQr} onOpenChange={setOpenQr}>
					<DialogContent>
						<DialogHeader>
							<DialogTitle className="text-center text-2xl">
								{me?.name}
							</DialogTitle>
						</DialogHeader>
						<div className="flex place-content-center py-6">
							<QRCode value={me?.address ?? ""} size={350} />
						</div>
						<DialogFooter>
							<Button className="w-full">Close</Button>
						</DialogFooter>
					</DialogContent>
				</Dialog>

				<Dialog open={openScanner} onOpenChange={setOpenScanner}>
					<DialogContent>
						<DialogHeader>
							<DialogTitle></DialogTitle>
						</DialogHeader>
						<Scanner
							allowMultiple={true}
							onScan={(result) => {
								const id = result.at(0)?.rawValue;
								if (typeof id === "string" && id.startsWith("0x")) {
									setReceiver(id);
									setOpenTransfer(true);
									setOpenScanner(false);
								}
							}}
						/>
					</DialogContent>
				</Dialog>

				<Dialog
					open={openTransfer}
					onOpenChange={(value) => {
						setTranferError("");
						setOpenTransfer(value);
					}}
				>
					<DialogContent>
						<DialogHeader>
							<DialogTitle>Transfer Point</DialogTitle>
						</DialogHeader>
						<Label>Token</Label>
						<Select value={tokenAddress} onValueChange={setTokenAddress}>
							<SelectTrigger className="w-full">
								<SelectValue placeholder="Select token" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={"SEL"}>SEL</SelectItem>
								<SelectItem value={"KHR"}>KHR</SelectItem>
								{createdTokens
									?.filter((token) => token.status === "CREATED")
									.map((token) => (
										<SelectItem
											value={token.token_address!}
											key={token.token_address!}
										>
											{token.symbol}
										</SelectItem>
									))}
							</SelectContent>
						</Select>
						<Label>Amount</Label>
						<Input
							type="number"
							value={amount}
							onChange={(e) => setAmount(Number(e.target.value))}
						/>
						<div>
							{!!transferError && (
								<p className="text-destructive">{transferError}</p>
							)}
						</div>
						<DialogFooter>
							<Button
								className="w-full"
								onClick={handleTransfer}
								disabled={amount === 0 || tokenAddress === "" || transfering}
							>
								{transfering && <Loader2 className="animate-spin" />}
								Send
							</Button>
						</DialogFooter>
					</DialogContent>
				</Dialog>
			</div>
		</div>
	);
}
