/// <reference types="vite/client" />

import { ColorModeProvider, ColorModeScript, cookieStorageManagerSSR } from "@kobalte/core";
import { TanStackDevtools } from "@tanstack/solid-devtools";
import { QueryClientProvider } from "@tanstack/solid-query";
import { SolidQueryDevtoolsPanel } from "@tanstack/solid-query-devtools";
import { createRootRouteWithContext, HeadContent, Outlet, Scripts } from "@tanstack/solid-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/solid-router-devtools";
import appCss from "~/app.css?url";
import { Toaster } from "~/components/ui/sonner";
import { Cookies } from "~/lib/cookies";
import { fetchUser } from "~/lib/user";
import type { RouterContext } from "~/router";

export const Route = createRootRouteWithContext<RouterContext>()({
    head: () => ({
        links: [{ rel: "stylesheet", href: appCss }],
    }),
    beforeLoad: async () => {
        const user = await fetchUser();
        return { user };
    },
    shellComponent: RootDocument,
});

function RootDocument() {
    const context = Route.useRouteContext();
    const storageManager = cookieStorageManagerSSR(Cookies.getRaw());

    return (
        <QueryClientProvider client={context().queryClient}>
            <HeadContent />

            <ColorModeScript storageType={storageManager.type} />
            <ColorModeProvider storageManager={storageManager}>
                <Outlet />
                <Toaster />
            </ColorModeProvider>

            <TanStackDevtools
                plugins={[
                    {
                        name: "TanStack Query",
                        render: <SolidQueryDevtoolsPanel />,
                    },
                    {
                        name: "TanStack Router",
                        render: <TanStackRouterDevtoolsPanel />,
                    },
                ]}
            />

            <Scripts />
        </QueryClientProvider>
    );
}
