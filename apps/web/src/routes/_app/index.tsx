import { createFileRoute } from "@tanstack/solid-router";
import { Show } from "solid-js";
import { ListPage } from "~/components/list-page";
import { fetchList } from "~/lib/list";

export const Route = createFileRoute("/_app/")({
    component: RouteComponent,
    ssr: false,
    loader: async () => {
        const detail = await fetchList("default");
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
