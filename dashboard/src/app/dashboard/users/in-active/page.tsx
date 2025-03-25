"use client";
import {
	Table,
	TableBody,
	TableCaption,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { getAllUsers } from "@/lib/api/admin/user";
import { useQuery } from "@tanstack/react-query";
import React from "react";
import EachUser from "../components/EachUser";

export default function InActiveUsers() {
	const { data } = useQuery({
		queryKey: ["in-active-users"],
		queryFn: getAllUsers,
	});

	return (
		<Table>
			<TableCaption>A list of your recent invoices.</TableCaption>
			<TableHeader>
				<TableRow>
					<TableHead>Name</TableHead>
					<TableHead>Email</TableHead>
					<TableHead>Address</TableHead>
					<TableHead>Balance</TableHead>
					<TableHead>Status</TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{data?.map((user, index) => (
					<EachUser key={index} user={user} />
				))}
			</TableBody>
		</Table>
	);
}
