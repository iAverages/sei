import { createFileRoute, Outlet } from "@tanstack/solid-router";
import { SidebarInset, SidebarProvider } from "~/components/ui/sidebar";
import { AppSidebar } from "~/components/ui/sidebar/app-sidebar";
import { SiteHeader } from "~/components/ui/sidebar/header";
import { cn } from "~/lib/utils";

export const Route = createFileRoute("/_app")({
    component: RouteComponent,
    staleTime: 60 * 1000,
    beforeLoad: () => {
        //
    },
});

function RouteComponent() {
    return (
        <SidebarProvider
            style={{
                "--sidebar-width": "calc(var(--spacing) * 72)",
                "--header-height": "calc(var(--spacing) * 12)",
            }}
        >
            <AppSidebar />
            <SidebarInset
                class={cn(
                    "mr-0 mb-0",
                    "md:peer-data-[variant=inset]:m-0 md:peer-data-[variant=inset]:mt-2 md:peer-data-[variant=inset]:rounded-none md:peer-data-[variant=inset]:rounded-tl-xl md:peer-data-[variant=inset]:shadow",
                    "md:peer-data-[state=collapsed]:peer-data-[variant=inset]:m-0  md:peer-data-[state=collapsed]:peer-data-[variant=inset]:rounded-none duration-200 transition-all",
                )}
            >
                <SiteHeader />
                <div class="flex flex-1 flex-col">
                    <div class="@container/main flex flex-1 flex-col gap-2 p-3">
                        <Outlet />
                    </div>
                </div>
            </SidebarInset>
        </SidebarProvider>
    );
}
