"use client";

import { SidebarIcon } from "lucide-react";

// import { SearchForm } from "@/components/search-form";
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbLink,
	BreadcrumbList,
	BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useSidebar } from "@/components/ui/sidebar";
import { usePathname } from "next/navigation";
import { Fragment } from "react";
import Link from "next/link";

export function SiteHeader() {
	const { toggleSidebar } = useSidebar();
	const pathname = usePathname();
	const segments = pathname.split("/").filter((item) => !!item);

	return (
		<header className="bg-background sticky top-0 z-50 flex w-full items-center border-b">
			<div className="flex h-(--header-height) w-full items-center gap-2 px-4">
				<Button
					className="h-8 w-8"
					variant="ghost"
					size="icon"
					onClick={toggleSidebar}
				>
					<SidebarIcon />
				</Button>
				<Separator orientation="vertical" className="mx-2 h-4" />
				<Breadcrumb className="hidden sm:block">
					<BreadcrumbList>
						{segments.map((segment, index, all) => {
							const path = all.slice(0, index + 1);
							return (
								<Fragment key={index}>
									<BreadcrumbItem>
										<BreadcrumbLink className="capitalize" asChild>
											<Link href={`/${path.join("/")}`}>{segment}</Link>
										</BreadcrumbLink>
									</BreadcrumbItem>
									{all.length - 1 > index && <BreadcrumbSeparator />}
								</Fragment>
							);
						})}
					</BreadcrumbList>
				</Breadcrumb>
				{/* <SearchForm className="w-full sm:ml-auto sm:w-auto" /> */}
			</div>
		</header>
	);
}
