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
import { RouteSectionProps } from "@solidjs/router";
import { createUser } from "~/components/auth";
import { createMutation } from "@tanstack/solid-query";
import { Button } from "~/components/ui/button";
import { AnimeList, ListStatus, RelatedAnime, useAnimeList } from "~/hooks/useAnimeList";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/accordion";
import { Card, CardHeader, CardTitle } from "~/components/ui/card";
import { Badge } from "~/components/ui/badge";

type AnimeCardProps = {
    grouped?: boolean;
    anime: AnimeList | RelatedAnime;
    getAnimeUserList: (id: number) => AnimeList | undefined;
    disabled?: boolean;
    showOverlayInfo?: boolean;
};

const InnerAnimeCard = (props: AnimeCardProps) => (
    <div
        class={"transition-transform flex flex-col items-center text-center"}
        classList={{
            "opacity-25": props.disabled,
            "pointer-events-none": props.disabled,
        }}>
        <img class={"h-[317px] w-[225px]"} draggable={false} src={props.anime.picture} />
        <p>{props.anime.romaji_title}</p>
    </div>
);

const AnimeCardComp = (props: AnimeCardProps) => {
    console.log("props", props.anime);
    return (
        <div class={"flex relative items-center justify-center w-full"}>
            <Show when={props.showOverlayInfo}>
                <div class={"absolute top-2 mr-2 flex w-full justify-end"}>
                    <Show
                        when={
                            typeof props.anime.relation !== "string" &&
                            props.anime.relation.filter((r) => isStatus(r, ["RELEASING", "NOT_YET_RELEASED"])).length >
                                0
                        }>
                        <Badge>New Season Soon</Badge>
                    </Show>
                    <Show when={!props.getAnimeUserList(props.anime.id)}>
                        <Badge>Not In List</Badge>
                    </Show>
                </div>
            </Show>
            <InnerAnimeCard {...props} />
            <Show when={props.grouped}>
                <For each={props.anime.relation}>
                    {(related) => <InnerAnimeCard anime={related} getAnimeUserList={props.getAnimeUserList} />}
                </For>
            </Show>
        </div>
    );
};

const AnimeCard = (props: AnimeCardProps) => {
    const sortable = createSortable(props.anime.id);
    const [state] = useDragDropContext();

    return (
        <div
            use:sortable
            class="sortable transition-opacity"
            classList={{
                "opacity-25 duration-250": sortable.isActiveDraggable || props.disabled,
                "transition-transform": !!state.active.draggable,
            }}>
            <AnimeCardComp grouped={false} {...props} />
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

// Helper function to convert a day of the week to a number (0 = Sunday, 6 = Saturday)
const dayOfWeekToNumber = (day: string): number => {
    const days = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];
    return days.indexOf(day?.toLowerCase());
};

// Converts broadcast time to a Date object in JST
const broadcastTimeToJSTDate = (broadcastDay: string, broadcastTime: string) => {
    const now = new Date();
    const dayIndex = dayOfWeekToNumber(broadcastDay);
    if (dayIndex === -1) {
        return null;
    }
    // Create a Date object for the next occurrence of the broadcast day
    let broadcastDate = new Date(now);
    broadcastDate.setDate(now.getDate() + ((7 + dayIndex - now.getDay()) % 7));
    const [hours, minutes] = broadcastTime.split(":").map(Number);
    // Set time for broadcast in JST (+9 UTC)
    broadcastDate.setHours(hours, minutes, 0, 0);
    return broadcastDate;
};

// Main function to check if the broadcast is within 12 hours of the user's current time
function isBroadcastWithin12Hours(item: AnimeList) {
    return false;
    // const broadcastDay = item.broadcast?.day_of_the_week;
    // const broadcastTime = item.broadcast?.start_time;

    // // Convert broadcast time to Date object in JST
    // const broadcastDate = broadcastTimeToJSTDate(broadcastDay, broadcastTime);
    // if (!broadcastDate) {
    //     return false;
    // }
    // // Convert broadcast Date to UTC
    // const broadcastUTC = broadcastDate.getTime() - 9 * 60 * 60 * 1000; // subtract 9 hours from JST to get UTC
    // // Adjust broadcast time to user's local time
    // const broadcastLocalTime = new Date(broadcastUTC + new Date().getTimezoneOffset() * -1 * 60000);

    // // Get current user time
    // const currentTime = new Date();
    // // Calculate difference in hours between broadcast time and current time
    // const diffHours = (broadcastLocalTime.getTime() - currentTime.getTime()) / (1000 * 60 * 60);

    // // Check if the difference is within ±12 hours
    // return Math.abs(diffHours) <= 12;
}
const c = () => {
    return [] as (AnimeList | RelatedAnime)[];
};

const getS1Anime = (anime: AnimeList | RelatedAnime) => {
    let base: AnimeList | RelatedAnime = anime;
    for (const r of anime.relation) {
        if (r.relation === "PREQUEL") {
            base = getS1Anime(r);
        }
    }

    if (base?.id === anime.id) {
        return anime;
    }

    return base;
};

const isStatus = (anime: AnimeList | RelatedAnime, status: string[]) => {
    return status.includes(anime.status);
};

export default function Home(props: RouteSectionProps) {
    const user = createUser();
    const anime = useAnimeList();

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
                    // if (!userList && isStatus(r, ["RELEASING", "NOT_YET_RELEASED"])) {
                    sequalNotInList.push(r);
                }

                if (a.watch_status === "COMPLETED" && userList?.watch_status !== "COMPLETED") {
                    hasWatchedPrequal.push(r);
                }

                const prev = a.relation[index - 1];
                const prevList = anime.data.animes.find((a) => a.id === prev?.id);
                if (prev.romaji_title.startsWith("Date A Live")) {
                    console.log("prev", prev);
                }

                if (
                    isStatus(r, ["NOT_YET_RELEASED", "RELEASING"]) &&
                    a.watch_status !== "COMPLETED" &&
                    (prevList?.watch_status !== "COMPLETED" || prevList === undefined)
                ) {
                    hasNotWatchedPrequal.push(prev);
                }

                if (
                    r.relation === "SEQUEL" &&
                    (r.status === "RELEASING" || r.status === "NOT_YET_RELEASED") &&
                    userList?.watch_status !== "WATCHING"
                ) {
                    if (r.romaji_title.startsWith("MASHLE")) {
                        console.log("r", userList);
                    }
                    upcomingSequals.push(r);
                }
            }
        }

        console.log("hasNotWatchedPrequal", hasNotWatchedPrequal);

        return {
            watchingReleasing,
            watchingReleased,
            notWatchingReleasing,
            hasSequel,
            hasWatchedPrequal,
            hasNotWatchedPrequal: hasNotWatchedPrequal,
            seasonOnes,
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

    // createEffect(() => {
    //     console.log("featured", filteredAnimes());
    // });

    const releasedAnime = createMemo(() => {
        const released = anime.data?.animes.filter((a) => a.status === "FINISHED" && a.watch_status !== "COMPLETED");
        return released;
    });

    const updateListOrder = createMutation(() => ({
        mutationKey: ["anime", "list", "update"],
        mutationFn: async (ids: number[]) => {
            console.log("ids", ids);
            const res = await fetch("http://localhost:3001/api/v1/order", {
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

    const [items, setItems] = createSignal(filteredAnimes().watchingReleased);

    createEffect(() => {
        setItems(filteredAnimes().watchingReleased);
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
        const a = anime.data?.animes.find((a) => a.id === id);
        console.log("a", a);
        return a;
    };

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
                <div>{anime.data.status}</div>
                <Show when={anime.data.status === ListStatus.Importing}>
                    <Card>
                        <CardHeader>
                            <CardTitle>We are importing your list</CardTitle>
                        </CardHeader>
                    </Card>
                </Show>

                {/* <Show when={anime.data.status !== ListStatus.Importing}> */}
                <Show when={anime.data.status === ListStatus.Updating}>
                    <div>We are updating your list.</div>
                </Show>

                <Accordion multiple={false} collapsible>
                    {/* <AccordionItem value={"a"}>
                        <AccordionTrigger>
                            <h1>a ({filteredAnimes().a.length})</h1>
                        </AccordionTrigger>

                        <AccordionContent>
                            <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                <For each={filteredAnimes().a}>
                                    {(anime) => <AnimeCardComp grouped anime={anime} />}
                                </For>
                            </div>
                        </AccordionContent>
                    </AccordionItem> */}
                    {/* <AccordionItem value={"not-watched-prequel"}>
                        <AccordionTrigger>
                            <h1>not-watched-prequel ({filteredAnimes().hasNotWatchedPrequal.length})</h1>
                        </AccordionTrigger>

                        <AccordionContent>
                            <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                <For each={filteredAnimes().hasNotWatchedPrequal}>
                                    {(anime) => <AnimeCardComp anime={anime} grouped />}
                                </For>
                            </div>
                        </AccordionContent>
                    </AccordionItem>

                    <AccordionItem value={"s ones"}>
                        <AccordionTrigger>
                            <h1>s ones ({filteredAnimes().seasonOnes.length})</h1>
                        </AccordionTrigger>

                        <AccordionContent>
                            <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                <For each={filteredAnimes().seasonOnes}>
                                    {(anime) => <AnimeCardComp anime={anime} grouped />}
                                </For>
                            </div>
                        </AccordionContent>
                    </AccordionItem> */}
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
                                                <AnimeCardComp
                                                    anime={anime}
                                                    getAnimeUserList={getAnimeUserList}
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
                {/* <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                    <For each={featuredAnime().releasingToday}>{(anime) => <AnimeCardComp anime={anime} />}</For>
                </div>
                <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                    <For each={featuredAnime().releasingUnwatched}>{(anime) => <AnimeCardComp anime={anime} />}</For>
                </div>
                <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                    <For each={featuredAnime().gettingNewSeason}>{(anime) => <AnimeCardComp anime={anime} />}</For>
                </div> */}

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
                                        showOverlayInfo
                                        getAnimeUserList={getAnimeUserList}
                                    />
                                )}
                            </For>
                        </div>
                    </SortableProvider>
                    <DragOverlay class={"transition-transform"}>
                        {(draggable) => (
                            <AnimeCardComp
                                anime={items().find((a) => a.id === draggable.id)}
                                getAnimeUserList={getAnimeUserList}
                            />
                        )}
                    </DragOverlay>
                </DragDropProvider>
            </Show>
            {/* </Show> */}
        </div>
    );
}
