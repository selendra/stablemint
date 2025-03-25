import { TableCell, TableRow } from "@/components/ui/table";
import React from "react";

export default function EachToken({ token }: { token: string }) {
	return (
		<TableRow>
			<TableCell className="font-mono font-normal">{token}</TableCell>
			<TableCell>{}</TableCell>
		</TableRow>
	);
}
