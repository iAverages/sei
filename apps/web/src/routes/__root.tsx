/// <reference types="vite/client" />

import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { ColorModeProvider, ColorModeScript, cookieStorageManagerSSR } from "@kobalte/core";
import appCss from "~/app.css?url";
import { SidebarInset, SidebarProvider } from "~/components/ui/sidebar";
import { AppSidebar } from "~/components/ui/sidebar/app-sidebar";
import { Skeleton } from "~/components/ui/skeleton";
import { Cookies } from "~/lib/cookies";
import { SiteHeader } from "~/components/ui/sidebar/header";
import { cn } from "~/lib/utils";

export const Route = createRootRoute({
    head: () => ({
        links: [{ rel: "stylesheet", href: appCss }],
    }),
    shellComponent: RootDocument,
});

function RootDocument() {
    const storageManager = cookieStorageManagerSSR(Cookies.getRaw());

    return (
        <>
            <ColorModeScript storageType={storageManager.type} />
            <ColorModeProvider storageManager={storageManager}>
                <Outlet />
            </ColorModeProvider>
        </>
    );
}
