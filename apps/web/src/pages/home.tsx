import { useDragDropContext } from "@thisbeyond/solid-dnd";
import {
    DragDropProvider,
    DragDropSensors,
    DragOverlay,
    SortableProvider,
    createSortable,
    closestCenter,
} from "@thisbeyond/solid-dnd";
import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { createMutation } from "@tanstack/solid-query";
import { Button } from "~/components/ui/button";
import { AnimeList, ListStatus, RelatedAnime, useAnimeList } from "~/hooks/useAnimeList";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/accordion";
import { Card, CardHeader, CardTitle } from "~/components/ui/card";
import { Badge } from "~/components/ui/badge";
import { cn } from "~/lib/utils";
import {
    ContextMenu,
    ContextMenuCheckboxItem,
    ContextMenuContent,
    ContextMenuGroup,
    ContextMenuGroupLabel,
    ContextMenuItem,
    ContextMenuPortal,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuShortcut,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from "~/components/ui/context-menu";
import { FaSolidArrowUpRightFromSquare } from "solid-icons/fa";

type AnimeCardProps = {
    grouped?: boolean;
    anime: AnimeList | (RelatedAnime & { watch_status?: string });
    getAnimeUserList: (id: number) => AnimeList | undefined;
    disabled?: boolean;
    showOverlayInfo?: boolean;
    hasNotWatchedPrequal: (id: number) => boolean;
};

const Note = (props: { children: string; class?: string }) => {
    return (
        <Badge class={cn("w-2 hover:w-36 max-w-fit overflow-hidden transition-all duration-700 group", props.class)}>
            <span class={"text-nowrap opacity-0 group-hover:opacity-100 transition-all"}>{props.children}</span>
        </Badge>
    );
};

const AnimeCardBadges = (props: AnimeCardProps) => {
    return (
        <div class={"absolute top-2 mr-2 flex w-full justify-end flex-col items-end gap-1 z-10"}>
            <Show when={props.anime.watch_status === "ON_HOLD"}>
                <Note class={"bg-yellow-300 hover:bg-yellow-300"}>On Hold</Note>
            </Show>
            <Show when={props.anime.watch_status === "DROPPED"}>
                <Note class={"bg-red-400 hover:bg-red-400"}>Dropped</Note>
            </Show>

            <Show
                when={
                    typeof props.anime.relation !== "string" &&
                    props.anime.relation.filter((r) => isStatus(r, ["RELEASING"])).length > 0
                }>
                <Note class={"bg-green-300 hover:bg-green-300"}>New Season Releasing</Note>
            </Show>
            <Show
                when={
                    typeof props.anime.relation !== "string" &&
                    props.anime.relation.filter((r) => isStatus(r, ["NOT_YET_RELEASED"])).length > 0
                }>
                <Note class={"bg-green-300 hover:bg-green-300"}>New Season Soon</Note>
            </Show>
            <Show when={props.hasNotWatchedPrequal(props.anime.id)}>
                <Note class={"bg-red-400 hover:bg-red-400"}>Prequel Unwatched</Note>
            </Show>
            <Show when={!props.getAnimeUserList(props.anime.id)}>
                <Note class={"bg-yellow-300 hover:bg-yellow-300"}>Not In List</Note>
            </Show>
        </div>
    );
};

const AnimeCardInnerContent = (props: AnimeCardProps) => {
    return (
        <div
            class={"flex"}
            classList={{
                "opacity-25": props.disabled,
                "pointer-events-none": props.disabled,
                "transition-transform": true,
            }}>
            <div
                class={"transition-transform flex flex-col items-center text-center "}
                classList={{
                    "opacity-25": props.disabled,
                    "pointer-events-none": props.disabled,
                }}>
                <div class={"relative"}>
                    <img class={"h-[317px] w-[225px]"} draggable={false} src={props.anime.picture} />
                    <p
                        class={
                            "absolute bottom-0 px-1 bg-slate-800 opacity-80 w-full min-h-12 py-2 flex items-center justify-center"
                        }>
                        <span class={"opacity-100"}>{props.anime.romaji_title}</span>
                    </p>
                </div>
            </div>
        </div>
    );
};

const AnimeCardInner = (props: AnimeCardProps) => {
    return (
        <div class={"flex relative items-center justify-center w-full"}>
            <Show when={props.showOverlayInfo}>
                <AnimeCardBadges {...props} />
            </Show>
            <AnimeCardInnerContent {...props} />
        </div>
    );
};

const AnimeCardWithContext = (props: AnimeCardProps & { isDraggable?: boolean; bringToFront: () => void }) => {
    return (
        <div class={"flex relative items-center justify-center w-full"}>
            <Show when={props.showOverlayInfo}>
                <AnimeCardBadges {...props} />
            </Show>
            <ContextMenu>
                <ContextMenuTrigger>
                    <AnimeCardInnerContent {...props} />
                </ContextMenuTrigger>
                <ContextMenuContent class="w-48">
                    <ContextMenuItem onClick={props.bringToFront}>
                        <span>Bring To Front</span>
                    </ContextMenuItem>
                    <ContextMenuSub overlap>
                        <ContextMenuSubTrigger>Watch Status</ContextMenuSubTrigger>
                        <ContextMenuPortal>
                            <ContextMenuGroup>
                                <ContextMenuRadioGroup value={props.anime.watch_status} onChange={() => {}}>
                                    <ContextMenuRadioItem value="WATCHING">Watching</ContextMenuRadioItem>
                                    <ContextMenuRadioItem value="COMPLETED">Completed</ContextMenuRadioItem>
                                    <ContextMenuRadioItem value="PLAN_TO_WATCH">Plan To Watch</ContextMenuRadioItem>
                                    <ContextMenuRadioItem value="DROPPED">Dropped</ContextMenuRadioItem>
                                </ContextMenuRadioGroup>
                            </ContextMenuGroup>
                        </ContextMenuPortal>
                    </ContextMenuSub>
                    <ContextMenuSeparator />
                    <ContextMenuItem class={"pointer-events-auto"}>
                        <a
                            class={"w-full"}
                            href={`https://myanimelist.net/anime/${props.anime.id}`}
                            target={"_blank"}
                            rel={"noreferrer noopener"}>
                            <span class={"flex justify-between w-full items-center"}>
                                View on MAL
                                <FaSolidArrowUpRightFromSquare />
                            </span>
                        </a>
                    </ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
        </div>
    );
};

const AnimeCard = (props: AnimeCardProps & { bringToFront: () => void }) => {
    const sortable = createSortable(props.anime.id);
    const [state] = useDragDropContext();

    return (
        <div
            use:sortable
            class="sortable transition-opacity touch-none"
            classList={{
                "opacity-25 duration-250": sortable.isActiveDraggable || props.disabled,
                "transition-transform": !!state.active.draggable,
            }}>
            <AnimeCardWithContext grouped={false} {...props} isDraggable />
        </div>
    );
};

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
    return [] as AnimeList[];
};

const isStatus = (anime: AnimeList | RelatedAnime, status: string[]) => {
    return status.includes(anime.status);
};

export default function Home() {
    const anime = useAnimeList();

    createEffect(() => {
        if (anime.data?.status === "importing") {
            setInterval(() => {
                anime.refetch();
            }, 1000);
        }
    });

    const filteredAnimes = createMemo(() => {
        if (!anime.data) {
            return {};
        }
        const d = new Date();
        const utc = d.getTime() + d.getTimezoneOffset() * 60000;
        const nd = new Date(utc + 3600000 * 9);
        const todayJp = new Date(nd).toLocaleString("en-US", { weekday: "long" }).toLowerCase();
        const today = new Date().toLocaleString("en-US", { weekday: "long" }).toLowerCase();
        console.log("today", today, todayJp);

        const watchingReleasing = c();
        const watchingReleased = c();
        const notWatchingReleasing = c();
        const hasSequel = c();
        const hasWatchedPrequal = c();
        const hasNotWatchedPrequal = c();
        const upcomingSequals = c();
        const sequalNotInList = c();

        const seasonOnes = anime.data.animes.filter((a) => {
            for (const r of a.relation) {
                if (r.relation === "PREQUEL") {
                    return false;
                }
            }

            return true;
        });

        for (const a of seasonOnes) {
            if (a.watch_status !== "COMPLETED" && a.status === "FINISHED") {
                watchingReleased.push(a);
            }

            if (a.watch_status !== "COMPLETED" && a.status === "RELEASING") {
                watchingReleasing.push(a);
            }

            let index = 0;
            for (const r of a.relation) {
                index++;
                if (r.relation === "SEQUEL") {
                    hasSequel.push(a);
                }

                const userList = anime.data.animes.find((a) => a.id === r.id);

                if (!userList) {
                    sequalNotInList.push(r as unknown as AnimeList);
                } else {
                    if (userList.watch_status !== "COMPLETED" && userList.status === "FINISHED") {
                        watchingReleased.push(userList);
                    }

                    if (userList.watch_status !== "COMPLETED" && userList.status === "RELEASING") {
                        watchingReleasing.push(userList);
                    }
                }

                if (a.watch_status === "COMPLETED" && userList?.watch_status !== "COMPLETED") {
                    hasWatchedPrequal.push(userList);
                }

                const prev = a.relation[index - 1];
                const prevList = anime.data.animes.find((a) => a.id === prev?.id);

                if (
                    isStatus(r, ["NOT_YET_RELEASED", "RELEASING"]) &&
                    a.watch_status !== "COMPLETED" &&
                    (prevList?.watch_status !== "COMPLETED" || prevList === undefined)
                ) {
                    hasNotWatchedPrequal.push(prevList);
                }

                if (
                    r.relation === "SEQUEL" &&
                    (r.status === "RELEASING" || r.status === "NOT_YET_RELEASED") &&
                    userList?.watch_status !== "WATCHING"
                ) {
                    upcomingSequals.push(userList || (r as unknown as AnimeList));
                }
            }
        }

        return {
            // watchingReleasing,
            watchingReleased,
            // notWatchingReleasing,
            // hasSequel,
            // hasWatchedPrequal,
            // hasNotWatchedPrequal: hasNotWatchedPrequal,
            // seasonOnes,
            upcomingSequals,
            sequalNotInList,
            // watching,
            // releasedIncomplete,
            // releasingSequals,
            // upcomingSequals,
            // hasUpcomingSequalsNotWatchedPrequal,
            // firstSeasonAnimes,
            // seasonsWithMultipleSeasons: animeWithMultipleSeasons,
            // unwatchedWithReleasingSequals,
            // hasSecondSeasonInListWithoutPrequal,
            // upcomingSequalsWithoutPrequalWatched,
        };
    });

    const updateListOrder = createMutation(() => ({
        mutationKey: ["anime", "list", "update"],
        mutationFn: async (ids: number[]) => {
            console.log("ids", ids);
            const res = await fetch(`${import.meta.env.VITE_API_URL ?? ""}/api/v1/order`, {
                method: "POST",
                credentials: "include",
                body: JSON.stringify({ ids }),
                headers: {
                    "Content-Type": "application/json",
                },
            });

            if (!res.ok) {
                throw res;
            }

            return res;
        },
    }));

    const [items, setItems] = createSignal(filteredAnimes().watchingReleased ?? []);

    createEffect(() => {
        if (filteredAnimes()?.watchingReleased)
            setItems(filteredAnimes().watchingReleased.sort((a, b) => a.watch_priority - b.watch_priority));
    });

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
        }
    };

    const getAnimeUserList = (id: number) => {
        return anime.data?.animes.find((a) => a.id === id);
    };

    const hasNotWatchedPrequal = (id: number) => {
        const topLevel = anime.data?.animes.find((a) => a.id === id);

        if (topLevel?.relation.filter((r) => r.relation === "PREQUEL").length === 0) {
            return false;
        }

        const related = anime.data?.animes.filter((a) => a.relation.find((r) => r.id === id));

        if (!topLevel && (!related || related.length === 0)) {
            return false;
        }

        for (const r of related) {
            if (id === 56876) {
                console.log("aaaaa", r);
            }
            if (r.watch_status !== "COMPLETED") {
                if (id === 56876) {
                    console.log("r", r);
                    console.log("topLevel", topLevel);
                }
                return true;
            }
        }
        return false;
    };

    createEffect(() => {
        console.log("items", items().length);
    });

    return (
        <div class={"p-6 flex flex-col gap-3"}>
            <Button
                onClick={async () => {
                    console.time("updated");
                    await updateListOrder.mutateAsync(items()?.map((i) => i.id));
                    console.timeEnd("updated");
                }}
                class={"bg-blue-500"}>
                Update List
            </Button>

            <Show when={anime.data}>
                <Show when={anime.data.status === ListStatus.Importing}>
                    <Card>
                        <CardHeader>
                            <CardTitle>We are importing your list</CardTitle>
                        </CardHeader>
                    </Card>
                </Show>

                <Show when={anime.data.status === ListStatus.Updating}>
                    <Card>
                        <CardHeader>
                            <CardTitle>We are updating your list.</CardTitle>
                        </CardHeader>
                    </Card>
                </Show>

                <Accordion multiple={false} collapsible>
                    <For each={Object.keys(filteredAnimes())}>
                        {(key) => (
                            <AccordionItem value={key}>
                                <AccordionTrigger>
                                    <h1>
                                        {key} ({filteredAnimes()[key].length})
                                    </h1>
                                </AccordionTrigger>

                                <AccordionContent>
                                    <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                        <For each={filteredAnimes()[key]}>
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
                </Accordion>

                <div class={"w-full h-1 bg-red-500"}></div>

                <DragDropProvider onDragEnd={onDragEnd} collisionDetector={closestCenter}>
                    <Fix /> {/* See definition */}
                    <DragDropSensors />
                    <SortableProvider ids={ids()}>
                        <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                            <For each={items()}>
                                {(anime) => (
                                    <AnimeCard
                                        anime={anime}
                                        disabled={updateListOrder.isPending}
                                        grouped
                                        showOverlayInfo
                                        getAnimeUserList={getAnimeUserList}
                                        hasNotWatchedPrequal={hasNotWatchedPrequal}
                                        bringToFront={() => {
                                            const index = items().findIndex((a) => a.id === anime.id);
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
                                getAnimeUserList={getAnimeUserList}
                                hasNotWatchedPrequal={hasNotWatchedPrequal}
                            />
                        )}
                    </DragOverlay>
                </DragDropProvider>
            </Show>
        </div>
    );
}
