import { DragDropDebugger, useDragDropContext } from "@thisbeyond/solid-dnd";
import {
    DragDropProvider,
    DragDropSensors,
    DragOverlay,
    SortableProvider,
    createSortable,
    closestCenter,
} from "@thisbeyond/solid-dnd";
import { For, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { Button } from "~/components/ui/button";
import { RouteSectionProps } from "@solidjs/router";
import { createUser } from "~/components/auth";
import { createQuery } from "@tanstack/solid-query";

type AnimeItem = {
    list_status: {};
    node: { id: number; title: string; main_picture: { large: string; medium: string } };
};
const AnimeCard = (props: { anime: AnimeItem }) => {
    const sortable = createSortable(props.anime.node.id);

    // const [state] = useDragDropContext();
    return (
        <div
            use:sortable
            class="sortable"
            classList={{
                "opacity-25": sortable.isActiveDraggable,
                // "transition-transform": !!state.active.draggable,
            }}>
            <div>
                <img class={"h-[317px] w-[225px]"} draggable={false} src={props.anime.node.main_picture.medium} />
                <p>{props.anime.node.title}</p>
            </div>
        </div>
    );
};

const useAnimeList = () => {
    return createQuery(() => ({
        queryKey: ["anime", "list"],
        queryFn: async () => {
            const res = await fetch("http://localhost:3001/api/v1/mal/anime", {
                credentials: "include",
            });

            if (!res.ok) {
                throw res;
            }
            const anime = (await res.json()) as {
                data: AnimeItem[];
                paging: { next: string; previous: string; current: string };
            };

            return anime;
        },
    }));
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

export default function Home(props: RouteSectionProps) {
    const user = createUser();
    const anime = useAnimeList();

    const [items, setItems] = createSignal(anime.data?.data);

    createEffect(() => {
        console.log("anime.data", anime.data);
        setItems(anime.data?.data);
    });

    const ids = () => items().map((item) => item.node.id);

    const [activeItem, setActiveItem] = createSignal(null);

    const onDragStart = ({ draggable }) => setActiveItem(draggable.id);

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
            <Show when={user.data} fallback={<>Loading user...</>}>
                <p>{JSON.stringify(user.data)}</p>
            </Show>

            <Show when={anime.data}>
                <DragDropProvider onDragStart={onDragStart} onDragEnd={onDragEnd} collisionDetector={closestCenter}>
                    <Fix />
                    {/* <DragDropDebugger /> */}
                    <DragDropSensors />
                    <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6"}>
                        <SortableProvider ids={ids()}>
                            <For each={items()} fallback={<>Loading anime...</>}>
                                {(anime) => <AnimeCard anime={anime} />}
                            </For>
                        </SortableProvider>
                    </div>

                    <DragOverlay>
                        {/* {(draggable) => <AnimeCard anime={items().find((a) => a.node.id === draggable.id)} />} */}
                        <div class="sortable">{activeItem()}</div>
                    </DragOverlay>
                </DragDropProvider>
            </Show>
        </div>
    );
}
