import { createFileRoute, Outlet, redirect, useRouter } from "@tanstack/solid-router";
import { createSignal, onCleanup, onMount, Show } from "solid-js";
import { SidebarInset, SidebarProvider } from "~/components/ui/sidebar";
import { type HeaderButtonsArea, HeaderButtonsProvider } from "~/components/ui/sidebar/buttons-portal";
import { Header } from "~/components/ui/sidebar/header";
import { AppSidebar } from "~/components/ui/sidebar/sidebar";
import { Cookies } from "~/lib/cookies";
import { fetchImportStatus } from "~/lib/list";
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

        return { user };
    },
    loader: async ({ context: { user } }) => {
        return {
            user,
            isImporting: await fetchImportStatus(),
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
    const data = Route.useLoaderData();
    const router = useRouter();
    const [isImporting, setIsImporting] = createSignal(data().isImporting);

    onMount(() => {
        if (!isImporting()) return;

        let timeout: ReturnType<typeof setTimeout>;
        let stopped = false;
        const checkImportStatus = async () => {
            try {
                if (!(await fetchImportStatus())) {
                    await router.invalidate();
                    setIsImporting(false);
                    return;
                }
            } catch (error) {
                console.error("Failed to check import status", error);
            }

            if (!stopped) timeout = setTimeout(checkImportStatus, 2000);
        };

        timeout = setTimeout(checkImportStatus, 2000);
        onCleanup(() => {
            stopped = true;
            clearTimeout(timeout);
        });
    });

    return (
        <Show when={!isImporting()} fallback={<ImportScreen />}>
            <HeaderButtonsProvider buttonsAreaRef={headerButtonsArea} setButtonsAreaRef={setHeaderButtonsArea}>
                <SidebarProvider
                    style={{
                        "--sidebar-width": "calc(var(--spacing) * 72)",
                        "--header-height": "calc(var(--spacing) * 12)",
                    }}
                >
                    <AppSidebar user={data().user} />
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
        </Show>
    );
}

function ImportScreen() {
    return (
        <main class="flex min-h-screen items-center justify-center bg-background px-6">
            <section class="w-full max-w-md text-center" aria-live="polite" aria-busy="true">
                <div class="mx-auto mb-8 flex size-20 items-center justify-center rounded-full border border-border bg-card shadow-sm">
                    <div class="size-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
                </div>
                <p class="mb-3 text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                    Setting up Sei
                </p>
                <h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">Your anime list is being imported</h1>
                <p class="mt-4 leading-7 text-muted-foreground">
                    This may take a few minutes. Your list will open automatically when every title is ready.
                </p>
            </section>
        </main>
    );
}
