import { useDragDropContext } from "@thisbeyond/solid-dnd";
import { DragDropProvider, DragDropSensors, DragOverlay, SortableProvider, closestCenter } from "@thisbeyond/solid-dnd";
import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Button } from "~/components/ui/button";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/accordion";
import { Card, CardHeader, CardTitle } from "~/components/ui/card";
import { isStatus, isWatchStatus } from "~/lib/status";
import { AnimeCard, AnimeCardInner } from "~/components/anime-card";
import { createUpdateListOrder } from "~/hooks/createUpdateListOrder";
import { useBeforeLeave } from "@solidjs/router";
import { Anime, AnimeReleaseStatus, ListStatus, UserListStatus, useAnimeList } from "~/hooks/useAnimeList";

// Fixes issue with being able to drag beyond some point
// im assuimg the images break the layout and solid-dnd doesnt
// pick it up for whatever reason
const Fix = () => {
    const [, { recomputeLayouts }] = useDragDropContext()!;

    let ticking = false;

    const update = () => {
        if (!ticking) {
            window.requestAnimationFrame(function () {
                recomputeLayouts();
                ticking = false;
            });

            ticking = true;
        }
    };

    onMount(() => {
        document.addEventListener("scroll", update);
    });

    onCleanup(() => {
        document.removeEventListener("scroll", update);
    });

    return null;
};

const c = () => {
    return [] as Anime[];
};

export default function Home() {
    const [hasReordered, setHasReordered] = createSignal(false);
    const userList = useAnimeList({ hasReordered });
    const updateListOrder = createUpdateListOrder();

    const getUserListMeta = (anime: Anime) => {
        const listStatus = userList.data?.list_status.find((l) => l.anime_id === anime.id);
        return listStatus;
    };

    const filteredAnimes = createMemo(() => {
        if (!userList.data) {
            return {};
        }

        const watchingReleasing = c();
        const watchingReleased = [] as Anime[];
        const notWatchingReleasing = c();
        const hasSequel = c();
        const hasWatchedPrequal = c();
        const hasNotWatchedPrequal = c();
        const upcomingSequals = c();
        const sequalNotInList = c();

        const seasonOnes = userList.data.animes.filter((anime) => {
            const animeListStatus = getUserListMeta(anime);

            if (animeListStatus?.status !== ListStatus.Completed && anime.status === AnimeReleaseStatus.Finished) {
                watchingReleased.push(anime);
            }

            if (animeListStatus?.status !== ListStatus.Completed && anime.status === AnimeReleaseStatus.Releasing) {
                watchingReleasing.push(anime);
            }

            const relations = userList.data.relations.filter((r) => r.anime_id === anime.id);

            for (const relatedAnime of relations) {
                if (relatedAnime.relation === "PREQUEL") {
                    return false;
                }
            }

            return true;
        });

        for (const a of seasonOnes) {
            let index = 0;
            // for (const r of a.relation) {
            //     index++;
            //     if (r.relation === "SEQUEL") {
            //         hasSequel.push(a);
            //     }

            //     const relationInUserList = anime.data.animes.find((a) => a.id === r.id);

            //     if (!relationInUserList) {
            //         sequalNotInList.push(r);
            //     }

            //     if (a.watch_status === "COMPLETED" && relationInUserList?.watch_status !== "COMPLETED") {
            //         hasWatchedPrequal.push(relationInUserList);
            //     }

            //     const prev = a.relation[index - 1];
            //     const prevList = anime.data.animes.find((a) => a.id === prev?.id);

            //     if (
            //         isStatus(r, ["NOT_YET_RELEASED", "RELEASING"]) &&
            //         !isWatchStatus(a, ["COMPLETED"]) &&
            //         // a.watch_status !== "COMPLETED"
            //         (prevList?.watch_status !== "COMPLETED" || prevList === undefined)
            //     ) {
            //         hasNotWatchedPrequal.push(prevList);
            //     }

            //     if (
            //         r.relation === "SEQUEL" &&
            //         (r.status === "RELEASING" || r.status === "NOT_YET_RELEASED") &&
            //         relationInUserList?.watch_status !== "WATCHING"
            //     ) {
            //         upcomingSequals.push(relationInUserList || (r as unknown as AnimeList));
            //     }
            // }
        }

        return {
            watchingReleased: watchingReleased.sort((a, b) => {
                const aStatus = userList.data.list_status.find((l) => l.anime_id === a.id);
                const bStatus = userList.data.list_status.find((l) => l.anime_id === b.id);

                return aStatus?.watch_priority - bStatus?.watch_priority;
            }),
            upcomingSequals,
            sequalNotInList,
        };
    });

    const [items, setItems] = createSignal(filteredAnimes().watchingReleased);
    createEffect(() => setItems(filteredAnimes().watchingReleased));
    const ids = () => items().map((item) => item.id);

    const onDragEnd = ({ draggable, droppable }) => {
        if (draggable && droppable) {
            const currentItems = items();
            const fromIndex = currentItems.findIndex((a) => a.id === draggable.id);
            const toIndex = currentItems.findIndex((a) => a.id === droppable.id);
            if (fromIndex !== toIndex) {
                const updatedItems = currentItems.slice();
                updatedItems.splice(toIndex, 0, ...updatedItems.splice(fromIndex, 1));
                setItems(updatedItems);
            }

            setHasReordered(
                items().some((anime, index) => {
                    const userListItem = getUserListMeta(anime);
                    if (userListItem?.watch_priority !== index + 1) {
                        return true;
                    }
                })
            );
        }
    };

    // const getAnimeUserList = (id: number) => {
    //     return userList.data?.animes.find((a) => a.id === id);
    // };

    // const hasNotWatchedPrequal = (id: number) => {
    //     const topLevel = userList.data?.animes.find((a) => a.id === id);

    //     if (topLevel?.relation.filter((r) => r.relation === "PREQUEL").length === 0) {
    //         return false;
    //     }

    //     const related = userList.data?.animes.filter((a) => a.relation.find((r) => r.id === id));

    //     if (!topLevel && (!related || related.length === 0)) {
    //         return false;
    //     }

    //     for (const r of related) {
    //         if (!isWatchStatus(r, ["COMPLETED"])) return true;
    //     }
    //     return false;
    // };

    useBeforeLeave((e) => {
        if (updateListOrder.isPending) {
            e.preventDefault();
            if (window.confirm("You have unsaved changes, are you sure you want to leave?")) {
                e.retry(true);
            }
            return;
        }
        if (hasReordered()) {
            e.preventDefault();
            if (window.confirm("You have unsaved changes, are you sure you want to leave?")) {
                e.retry(true);
            }
        }
    });

    return (
        <div class={"p-6 flex flex-col gap-3"}>
            <Button onClick={() => updateListOrder.mutate(items()?.map((i) => i.id))} class={"bg-blue-500"}>
                Update List
            </Button>
            <a href={"/login"}>login page</a>

            <Show when={userList.data}>
                <Show when={userList.data.import_status === UserListStatus.Importing}>
                    <Card>
                        <CardHeader>
                            <CardTitle>We are importing your list</CardTitle>
                        </CardHeader>
                    </Card>
                </Show>

                <Show when={userList.data.import_status === UserListStatus.Updating}>
                    <Card>
                        <CardHeader>
                            <CardTitle>We are updating your list.</CardTitle>
                        </CardHeader>
                    </Card>
                </Show>

                {/* <Accordion multiple={false} collapsible>
                    <For
                        each={[
                            { key: "upcomingSequals", label: "Upcoming Sequals" },
                            { key: "sequalNotInList", label: "Sequals Not In List" },
                        ]}>
                        {(group) => (
                            <AccordionItem value={group.key}>
                                <AccordionTrigger>
                                    <h1>
                                        {group.label} ({filteredAnimes()[group.key].length})
                                    </h1>
                                </AccordionTrigger>

                                <AccordionContent>
                                    <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                        <For each={filteredAnimes()[group.key]}>
                                            {(anime) => (
                                                <AnimeCardInner
                                                    anime={anime}
                                                    getAnimeUserList={getAnimeUserList}
                                                    hasNotWatchedPrequal={hasNotWatchedPrequal}
                                                    showOverlayInfo
                                                />
                                            )}
                                        </For>
                                    </div>
                                </AccordionContent>
                            </AccordionItem>
                        )}
                    </For>
                </Accordion> */}

                <div class={"w-full h-1 bg-red-500"}></div>

                <DragDropProvider onDragEnd={onDragEnd} collisionDetector={closestCenter}>
                    <Fix /> {/* See definition */}
                    <DragDropSensors />
                    <SortableProvider ids={ids()}>
                        <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-8 gap-3"}>
                            <For each={items()}>
                                {(animeItem) => (
                                    <AnimeCard
                                        anime={animeItem}
                                        listStatus={getUserListMeta(animeItem)}
                                        disabled={updateListOrder.isPending || userList.isRefetching}
                                        grouped
                                        showOverlayInfo
                                        bringToFront={() => {
                                            const index = items().findIndex((a) => a.id === animeItem.id);
                                            const updatedItems = items().slice();
                                            updatedItems.splice(0, 0, ...updatedItems.splice(index, 1));
                                            setItems(updatedItems);
                                        }}
                                    />
                                )}
                            </For>
                        </div>
                    </SortableProvider>
                    <DragOverlay class={"transition-transform"}>
                        {(draggable) => (
                            <AnimeCardInner
                                anime={items().find((a) => a.id === draggable.id)}
                                // getAnimeUserList={getAnimeUserList}
                            />
                        )}
                    </DragOverlay>
                </DragDropProvider>
            </Show>
        </div>
    );
}
