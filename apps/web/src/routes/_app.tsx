import { createFileRoute, Outlet, redirect } from "@tanstack/solid-router";
import { createSignal } from "solid-js";
import { SidebarInset, SidebarProvider } from "~/components/ui/sidebar";
import { type HeaderButtonsArea, HeaderButtonsProvider } from "~/components/ui/sidebar/buttons-portal";
import { Header } from "~/components/ui/sidebar/header";
import { AppSidebar } from "~/components/ui/sidebar/sidebar";
import { Cookies } from "~/lib/cookies";
import { prependSlash } from "~/lib/utils";

const handleGoToRedirect = () => {
    const goto = Cookies.get("goto");
    if (!goto) return;
    Cookies.set("goto", "", { maxAge: 0 });
    throw redirect({
        to: prependSlash(goto),
    });
};

export const Route = createFileRoute("/_app")({
    component: RouteComponent,
    staleTime: 60 * 1000,
    beforeLoad: ({ context: { user }, location }) => {
        if (!user) {
            Cookies.set("goto", location.href, { maxAge: 0 });
            throw redirect({
                to: "/login",
            });
        }

        handleGoToRedirect();
    },
    loader: () => {
        return {
            crumb: "List",
            data: [
                { id: "default", name: "Default" },
                { id: "movies", name: "Movies" },
            ],
        };
    },
});

function RouteComponent() {
    // TODO: can we speed this up, it takes a second to actually show the buttons
    const [headerButtonsArea, setHeaderButtonsArea] = createSignal<HeaderButtonsArea>(null);

    return (
        <HeaderButtonsProvider buttonsAreaRef={headerButtonsArea} setButtonsAreaRef={setHeaderButtonsArea}>
            <SidebarProvider
                style={{
                    "--sidebar-width": "calc(var(--spacing) * 72)",
                    "--header-height": "calc(var(--spacing) * 12)",
                }}
            >
                <AppSidebar />
                <SidebarInset>
                    <Header setButtonsAreaRef={setHeaderButtonsArea} />
                    <div class="flex flex-1 flex-col">
                        <div class="@container/main flex flex-1 flex-col gap-2 p-3">
                            <Outlet />
                        </div>
                    </div>
                </SidebarInset>
            </SidebarProvider>
        </HeaderButtonsProvider>
    );
}
