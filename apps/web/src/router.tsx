import { QueryClient } from "@tanstack/solid-query";
import { createRouter } from "@tanstack/solid-router";
import superjson from "superjson";
import { routeTree } from "./routeTree.gen";

declare module "solid-js" {
    namespace JSX {
        interface Directives {
            draggable: boolean;
            droppable: boolean;
            sortable: boolean;
        }
    }
}

export const createRouterContext = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            dehydrate: { serializeData: superjson.serialize },
            hydrate: { deserializeData: superjson.deserialize },
        },
    });

    return {
        queryClient,
    };
};
export type RouterContext = ReturnType<typeof createRouterContext>;

export function getRouter() {
    const router = createRouter({
        routeTree,
        defaultPreload: "intent",
        defaultErrorComponent: () => (
            <main class="flex min-h-screen items-center justify-center bg-background px-6 text-center text-foreground">
                <div>
                    <h1 class="text-2xl font-semibold">Something went wrong</h1>
                    <p class="mt-2 text-sm text-muted-foreground">The requested page could not be loaded.</p>
                </div>
            </main>
        ),
        defaultNotFoundComponent: () => (
            <main class="flex min-h-screen items-center justify-center bg-background px-6 text-center text-foreground">
                <div>
                    <h1 class="text-2xl font-semibold">Page not found</h1>
                    <p class="mt-2 text-sm text-muted-foreground">This page does not exist or is not available.</p>
                </div>
            </main>
        ),
        scrollRestoration: true,
        context: createRouterContext(),
        defaultOnCatch(error) {
            console.error("uncaught error", error.message, error.stack);
        },
    });

    return router;
}
