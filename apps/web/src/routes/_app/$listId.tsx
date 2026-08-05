import { createFileRoute, redirect } from "@tanstack/solid-router";
import { Show } from "solid-js";
import { ListPage } from "~/components/list-page";
import { fetchList } from "~/lib/list";

export const Route = createFileRoute("/_app/$listId")({
    component: RouteComponent,
    ssr: false,
    beforeLoad: ({ params }) => {
        if (params.listId === "default") throw redirect({ to: "/" });
    },
    loader: async ({ params }) => {
        const detail = await fetchList(params.listId);
        return { ...detail, crumb: detail.list.name };
    },
});

function RouteComponent() {
    const data = Route.useLoaderData();
    return (
        <Show when={data()} keyed>
            {(detail) => <ListPage detail={detail} />}
        </Show>
    );
}
