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
import { createMutation, createQuery } from "@tanstack/solid-query";
import { Button } from "~/components/ui/button";
import { Anime, AnimeList, ListStatus, useAnimeList } from "~/hooks/useAnimeList";

const AnimeCardComp = (props: { anime: AnimeList; disabled?: boolean }) => (
    <div
        class={"transition-transform flex flex-col items-center text-center"}
        classList={{
            "opacity-25": props.disabled,
            "pointer-events-none": props.disabled,
        }}>
        <img class={"h-[317px] w-[225px]"} draggable={false} src={props.anime.node.main_picture.medium} />
        <p>{props.anime.node.title}</p>
    </div>
);

const AnimeCard = (props: { anime: AnimeList; disabled?: boolean }) => {
    const sortable = createSortable(props.anime.node.id);
    const [state] = useDragDropContext();

    return (
        <div
            use:sortable
            class="sortable transition-opacity"
            classList={{
                "opacity-25 duration-250": sortable.isActiveDraggable || props.disabled,
                "transition-transform": !!state.active.draggable,
            }}>
            <AnimeCardComp anime={props.anime} />
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
    const broadcastDay = item.node.broadcast?.day_of_the_week;
    const broadcastTime = item.node.broadcast?.start_time;

    // Convert broadcast time to Date object in JST
    const broadcastDate = broadcastTimeToJSTDate(broadcastDay, broadcastTime);
    if (!broadcastDate) {
        return false;
    }
    // Convert broadcast Date to UTC
    const broadcastUTC = broadcastDate.getTime() - 9 * 60 * 60 * 1000; // subtract 9 hours from JST to get UTC
    // Adjust broadcast time to user's local time
    const broadcastLocalTime = new Date(broadcastUTC + new Date().getTimezoneOffset() * -1 * 60000);

    // Get current user time
    const currentTime = new Date();
    // Calculate difference in hours between broadcast time and current time
    const diffHours = (broadcastLocalTime.getTime() - currentTime.getTime()) / (1000 * 60 * 60);

    // Check if the difference is within ±12 hours
    return Math.abs(diffHours) <= 12;
}

export default function Home(props: RouteSectionProps) {
    const user = createUser();
    const anime = useAnimeList();

    const featuredAnime = createMemo(() => {
        const d = new Date();
        const utc = d.getTime() + d.getTimezoneOffset() * 60000;
        const nd = new Date(utc + 3600000 * 9);
        const todayJp = new Date(nd).toLocaleString("en-US", { weekday: "long" }).toLowerCase();
        const today = new Date().toLocaleString("en-US", { weekday: "long" }).toLowerCase();
        console.log("today", today, todayJp);

        const releasingToday = anime.data?.animes.filter(
            (a) => isBroadcastWithin12Hours(a) && a.node.status === "currently_airing"
        );
        const releasingUnwatched = anime.data?.animes.filter((a) => {
            return a.list_status.num_episodes_watched < a.node.num_episodes && a.node.status === "currently_airing";
        });

        return {
            releasingToday,
            releasingUnwatched,
        };
    });

    const releasedAnime = createMemo(() => {
        const released = anime.data?.animes.filter((a) => a.node.status === "finished_airing");
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

    const [items, setItems] = createSignal(releasedAnime());

    createEffect(() => {
        console.log("anime.data", anime.data);
        setItems(releasedAnime());
    });

    const ids = () => items().map((item) => item.node.id);

    const onDragEnd = ({ draggable, droppable }) => {
        if (draggable && droppable) {
            const currentItems = items();
            const fromIndex = currentItems.findIndex((a) => a.node.id === draggable.id);
            const toIndex = currentItems.findIndex((a) => a.node.id === droppable.id);
            if (fromIndex !== toIndex) {
                const updatedItems = currentItems.slice();
                updatedItems.splice(toIndex, 0, ...updatedItems.splice(fromIndex, 1));
                setItems(updatedItems);
            }
        }
    };

    return (
        <div class={"p-6 flex flex-col gap-3"}>
            <Button
                onClick={async () => {
                    console.time("updated");
                    await updateListOrder.mutateAsync(items()?.map((i) => i.node.id));
                    console.timeEnd("updated");
                }}
                class={"bg-blue-500"}>
                Update List
            </Button>

            <Show when={anime.data}>
                <div>{anime.data.status}</div>
                <Show when={anime.data.status === ListStatus.Importing}>
                    <div>We are importing your list</div>
                </Show>

                <Show when={anime.data.status !== ListStatus.Importing}>
                    <Show when={anime.data.status === ListStatus.Updating}>
                        <div>We are updating your list.</div>
                    </Show>
                    <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                        <For each={featuredAnime().releasingToday}>{(anime) => <AnimeCardComp anime={anime} />}</For>
                    </div>
                    <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                        <For each={featuredAnime().releasingUnwatched}>
                            {(anime) => <AnimeCardComp anime={anime} />}
                        </For>
                    </div>

                    <div class={"w-full h-1 bg-red-500"}></div>

                    <DragDropProvider onDragEnd={onDragEnd} collisionDetector={closestCenter}>
                        <Fix /> {/* See definition */}
                        <DragDropSensors />
                        <SortableProvider ids={ids()}>
                            <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"}>
                                <For each={items()}>
                                    {(anime) => <AnimeCard anime={anime} disabled={updateListOrder.isPending} />}
                                </For>
                            </div>
                        </SortableProvider>
                        <DragOverlay class={"transition-transform"}>
                            {(draggable) => <AnimeCardComp anime={items().find((a) => a.node.id === draggable.id)} />}
                        </DragOverlay>
                    </DragDropProvider>
                </Show>
            </Show>
        </div>
    );
}
