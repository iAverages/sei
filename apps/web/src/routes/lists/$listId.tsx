import { createFileRoute, notFound } from "@tanstack/solid-router";
import { For, Show } from "solid-js";
import { AnimeCard } from "~/components/anime-card";
import { fetchPublicList, ListApiError } from "~/lib/list";

export const Route = createFileRoute("/lists/$listId")({
    loader: async ({ params }) => {
        try {
            return { ...(await fetchPublicList(params.listId)), crumb: undefined };
        } catch (error) {
            if (error instanceof ListApiError && error.status === 404) throw notFound();
            throw error;
        }
    },
    component: PublicListPage,
    notFoundComponent: UnavailableListPage,
});

function UnavailableListPage() {
    return (
        <main class="flex min-h-screen items-center justify-center bg-background px-6 text-center text-foreground">
            <div>
                <h1 class="text-2xl font-semibold">List not available</h1>
                <p class="mt-2 text-sm text-muted-foreground">This list is private or no longer exists.</p>
            </div>
        </main>
    );
}

function PublicListPage() {
    const detail = Route.useLoaderData();

    return (
        <main class="min-h-screen bg-background px-3 py-8 text-foreground sm:px-6 lg:px-10">
            <header class="mx-auto mb-8 max-w-screen-2xl border-b border-border pb-6">
                <h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">{detail().list.name}</h1>
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
