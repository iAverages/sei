import { createFileRoute } from "@tanstack/solid-router";
import { For, Show } from "solid-js";
import { AnimeCard } from "~/components/anime-card";
import { fetchPublicList } from "~/lib/list";

export const Route = createFileRoute("/lists/$listId")({
    loader: async ({ params }) => ({ ...(await fetchPublicList(params.listId)), crumb: undefined }),
    component: PublicListPage,
});

function PublicListPage() {
    const detail = Route.useLoaderData();
    const visibility = () => (detail().list.visibility === "PUBLIC" ? "Public" : "Unlisted");

    return (
        <main class="min-h-screen bg-background px-3 py-8 text-foreground sm:px-6 lg:px-10">
            <header class="mx-auto mb-8 max-w-screen-2xl border-b border-border pb-6">
                <a href="/" class="text-sm font-semibold tracking-wide text-muted-foreground">
                    Sei
                </a>
                <div class="mt-4 flex flex-wrap items-end justify-between gap-3">
                    <h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">{detail().list.name}</h1>
                    <span class="rounded-full border border-border bg-muted px-3 py-1 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                        {visibility()}
                    </span>
                </div>
            </header>
            <section class="mx-auto max-w-screen-2xl">
                <Show
                    when={detail().anime.length > 0}
                    fallback={
                        <div class="flex min-h-72 items-center justify-center rounded-lg border border-dashed border-border px-6 text-center">
                            <div>
                                <h2 class="text-xl font-semibold">This list is empty</h2>
                                <p class="mt-2 text-sm text-muted-foreground">There are no anime to show yet.</p>
                            </div>
                        </div>
                    }
                >
                    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8">
                        <For each={detail().anime}>{(anime) => <AnimeCard anime={anime} />}</For>
                    </div>
                </Show>
            </section>
        </main>
    );
}
