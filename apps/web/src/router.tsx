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
        defaultErrorComponent: (err) => <p>{err.error.stack}</p>,
        defaultNotFoundComponent: () => <p>not found</p>,
        scrollRestoration: true,
        context: createRouterContext(),
    });

    return router;
}
