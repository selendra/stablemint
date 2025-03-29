import React, { ReactNode } from "react";

export default function ProfileLayOut({ children }: { children: ReactNode }) {
	return (
		<div className="w-full h-dvh overflow-hidden bg-muted">
			<div className="w-full max-w-lg h-full overflow-x-hidden overflow-y-auto bg-background mx-auto">
				{children}
			</div>
		</div>
	);
}
