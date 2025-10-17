import { createWindowVirtualizer } from "@tanstack/solid-virtual";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useMutation } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import {
    closestCenter,
    DragDropProvider,
    DragDropSensors,
    type DragEventHandler,
    DragOverlay,
    SortableProvider,
    useDragDropContext,
} from "@thisbeyond/solid-dnd";
import { type Accessor, batch, createEffect, createSignal, For, onMount, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { Motion, Presence } from "solid-motionone";
import { toast } from "solid-sonner";
import { AnimeCard, DraggableAnimeCard } from "~/components/anime-card";
import { BackToTop } from "~/components/back-to-top";
import { Alert, AlertTitle } from "~/components/ui/alert";
import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "~/components/ui/alert-dialog";
import { Button } from "~/components/ui/button";
import { HeaderButtonsPortal } from "~/components/ui/sidebar/buttons-portal";
import { type Anime, type AnimeListStatus, fetchList, updateListOrder } from "~/lib/list";
import { useSseStream } from "~/lib/sse";
import { moveIndexToStart } from "~/lib/utils";

export const Route = createFileRoute("/_app/$listId")({
    component: RouteComponent,
    ssr: false,
    loader: async () => {
        const { anime, isImporting } = await fetchList();
        return {
            crumb: "Default",
            anime,
            isImporting,
        };
    },
});
const useArrayStore = <TData, TId>(defaultValue: TData[], opts: { getId: (data: TData) => TId }) => {
    const getIds = (data: TData[]) => data.map(opts.getId);
    // only data is reactive, ids is used to checking if values exist in data but faster and
    // does not need to be reactive
    const [data, setData] = createStore(defaultValue);
    // ids is an array since we need the item ids as an array for the dnd components
    // and do not want to recreate this every time an anime is added (during first import
    // animes get added a lot this causes the page to become laggy for some time)
    const [ids, setIds] = createStore(getIds(data));
    // however we want a fast lookup for animes we  have stored, we use a map which is
    // itemId -> index in storage for faster lookups
    const idsMap = new Map(ids.map((id, index) => [id, index]));

    const append = (newData: TData) => {
        replace([newData]);
        return;
        const id = opts.getId(newData);
        if (idsMap.has(id)) return;
        batch(() => {
            setData(data.length, newData);
            idsMap.set(id, data.length);
            setIds(ids.length, id);
        });
    };

    const remove = (index: number) => {
        batch(() => {
            setData((prev) => {
                const removed = prev.splice(index, 1);
                removed.forEach((data) => {
                    const dataId = opts.getId(data);
                    const dataIndex = idsMap.get(dataId);
                    console.assert(dataIndex !== undefined, "failed to find dataId in idsMap", dataId);
                    idsMap.delete(dataId);
                    setIds((prev) => {
                        prev.splice(dataIndex!, 1);
                        return prev;
                    });
                });
                return prev;
            });
        });
    };

    const replace = (data: TData[]) => {
        batch(() => {
            setData(data);
            idsMap.clear();
            const newIds = getIds(data);
            setIds(newIds);
            newIds.forEach((id, index) => void idsMap.set(id, index));
        });
    };

    return {
        ids,
        idsMap,
        data,
        append,
        remove,
        replace,
    };
};

const computedStyle = window.getComputedStyle(document.documentElement);
const fontSize = computedStyle.getPropertyValue("font-size");

const getRemPixels = (rem: number) => {
    return Number.parseFloat(fontSize) * rem;
};

function RouteComponent() {
    const data = Route.useLoaderData();
    const [isImporting, setIsImporting] = createSignal(data().isImporting);
    const [hasReordered, setHasReordered] = createSignal(false);
    let initalAnime = [...data().anime];
    const {
        data: animes,
        ids: animeIdsArray,
        idsMap: animeIds,
        append: addAnime,
        replace: setAnimes,
    } = useArrayStore([...data().anime], { getId: (data) => data.id });

    type ListSseEvent = {
        anime: Anime;
    };
    useSseStream<ListSseEvent>({
        url: "/api/v1/user/list/sse",
        onMessage: ({ anime }) => {
            if (animeIds.has(anime.id)) {
                console.log("anime already in list, skipping");
                return;
            }
            console.log("got event", anime);
            initalAnime[initalAnime.length - 1] = anime;
            addAnime(anime);
            // virtualizer.measure();
        },
    });

    // biome-ignore lint/style/useConst: used for ref to dom node
    let gridRef: HTMLDivElement = null!;
    const lanes = 8;
    const gap = getRemPixels(0.75);
    const virtualizer = createWindowVirtualizer({
        get count() {
            return animes.length;
        },
        lanes,
        estimateSize: () => 317,
        gap,
        overscan: 5,
        scrollMargin: gridRef?.offsetTop ?? 0,
    });

    // const hasReordered = () => {
    //     return false;
    // return !animes.every((animeA, index) => initalAnime[index].id === animeA.id);
    // };

    // "hack" to not update hasReordered when adding anime during first import
    // if this is true then we will update hasReordered if anime changes
    let justDragged = false;
    createEffect(() => {
        if (!justDragged) {
            console.log("anime updated but we didnt drag");
            return;
        }
        console.log("animes updated");
        const reordered = !animes.every((animeA, index) => initalAnime[index].id === animeA.id);
        setHasReordered(reordered);
        justDragged = false;
    });

    const onDragEnd: DragEventHandler = (event) => {
        if (!event.droppable) return;
        const draggingAnime = animes.findIndex((anime) => anime.id === event.draggable.id);
        if (draggingAnime === -1) {
            console.warn("unable to find dragging anime", event.draggable.id);
            return;
        }

        const droppingAnime = animes.findIndex((anime) => anime.id === event.droppable!.id);
        if (droppingAnime === -1) {
            console.warn("unable to find dropping anime", event.draggable.id);
            return;
        }

        // no need to update if no items actually moved
        if (droppingAnime === draggingAnime) return;
        justDragged = true;
        const updatedItems = animes.slice();
        updatedItems.splice(droppingAnime, 0, ...updatedItems.splice(draggingAnime, 1));
        setAnimes(updatedItems);
        virtualizer.measure();
    };

    const updateListOrderMutation = useMutation(() => ({
        mutationKey: ["list", "update"],
        mutationFn: async (ids: number[]) => {
            await updateListOrder(ids);
            initalAnime = ids.map((id) => animes.find((a) => a.id === id)!);
            toast("List order saved successfully");
        },
    }));

    const updateAnimeStatus = useMutation(() => ({
        mutationKey: ["anime", "status", "update"],
        mutationFn: async ({ animeId, status }: { animeId: number; status: AnimeListStatus }) => {},
    }));

    return (
        <fieldset
            disabled={updateListOrderMutation.isPending || updateAnimeStatus.isPending}
            class="flex flex-col gap-2"
        >
            <BackToTop />
            <HeaderButtonsPortal>
                <Motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
                    <div class="flex gap-2">
                        <Button
                            disabled={!hasReordered()}
                            onClick={() => updateListOrderMutation.mutate(animeIdsArray)}
                        >
                            Save List Order
                        </Button>
                        <ResetButton hasReordered={hasReordered} reset={() => setAnimes([...initalAnime])} />
                    </div>
                </Motion.div>
            </HeaderButtonsPortal>

            <Presence>
                <Show when={isImporting()}>
                    <Motion.div
                        initial={{ opacity: 0, height: "0px" }}
                        animate={{ opacity: 1, height: "auto" }}
                        exit={{ opacity: 0, height: "0px" }}
                        transition={{ duration: 0.25 }}
                    >
                        <Alert variant={"destructive"}>
                            <AlertTitle>
                                Some animes in this list are still being imported. They should automatically appear once
                                complete.
                            </AlertTitle>
                        </Alert>
                    </Motion.div>
                </Show>
            </Presence>
            <Motion.div initial={{ opacity: 0, y: 50 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.25 }}>
                <DragDropProvider onDragEnd={onDragEnd} collisionDetector={closestCenter}>
                    <ScrollDragFix />
                    <DragDropSensors />
                    <SortableProvider ids={animeIdsArray}>
                        <div ref={gridRef}>
                            <div
                                style={{
                                    height: `${virtualizer.getTotalSize()}px`,
                                    width: "100%",
                                    position: "relative",
                                }}
                                class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-3"
                            >
                                <For each={virtualizer.getVirtualItems()}>
                                    {(item) => {
                                        console.log({ animes });
                                        const anime = animes[item.index]!;
                                        return (
                                            <div
                                                ref={(el) => queueMicrotask(() => virtualizer.measureElement(el))}
                                                data-index={item.index.toString()}
                                                style={{
                                                    position: "absolute",
                                                    top: 0,
                                                    left: `${(100 / lanes) * item.lane}% `,
                                                    // left: `calc(${(100 / lanes) * item.lane}% - ${gap / 2}px)`,
                                                    width: `calc(${100 / lanes}% - ${gap}px)`,
                                                    height: `${item.size}px`,
                                                    transform: `translateY(${item.start}px)`,
                                                }}
                                            >
                                                <DraggableAnimeCard
                                                    index={item.index}
                                                    anime={anime}
                                                    bringToFront={() => setAnimes(moveIndexToStart(animes, item.index))}
                                                    setStatus={(status) =>
                                                        updateAnimeStatus.mutate({ animeId: anime.id, status })
                                                    }
                                                />
                                            </div>
                                        );
                                    }}
                                </For>
                            </div>
                        </div>
                    </SortableProvider>
                    <DragOverlay class={"transition-transform"}>
                        {(draggable) => {
                            return <AnimeCard anime={animes.find((a) => a.id === draggable!.id)!} />;
                        }}
                    </DragOverlay>
                </DragDropProvider>
            </Motion.div>
        </fieldset>
    );
}

const ResetButton = (props: { hasReordered: Accessor<boolean>; reset: () => void }) => {
    const [open, setOpen] = createSignal(false);
    const handleAccept = async () => {
        props.reset();
        setOpen(false);
    };

    const handleDeny = () => {
        setOpen(false);
    };

    return (
        <AlertDialog open={open()} onOpenChange={setOpen}>
            <AlertDialogTrigger as={Button} disabled={!props.hasReordered()}>
                Reset
            </AlertDialogTrigger>

            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                    <AlertDialogDescription>
                        This action cannot be undone. This will reset any changes you have made to this list since the
                        last save.
                    </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter class="flex justify-between w-full sm:justify-between md:justify-between">
                    <Button variant={"destructive"} onClick={handleAccept}>
                        Yes, reset list order
                    </Button>
                    <Button onClick={handleDeny}>No, do not reset list order</Button>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    );
};

// Fixes issue with being able to drag beyond some point
// im assuimg the images break the layout and solid-dnd doesnt
// pick it up for whatever reason
const ScrollDragFix = () => {
    const [, { recomputeLayouts }] = useDragDropContext()!;

    let ticking = false;

    const update = () => {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                recomputeLayouts();
                ticking = false;
            });

            ticking = true;
        }
    };

    onMount(() => {
        makeEventListener(document, "scroll", update);
    });

    return null;
};
