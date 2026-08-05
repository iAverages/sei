import { createFileRoute, redirect } from "@tanstack/solid-router";

export const Route = createFileRoute("/_app/")({
    beforeLoad: () => {
        throw redirect({ to: "/$listId", params: { listId: "default" } });
    },
});
