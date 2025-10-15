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
import { type Accessor, createEffect, createSignal, For, onMount, Show } from "solid-js";
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
import { Anime, type AnimeListStatus, fetchList, updateListOrder } from "~/lib/list";
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

function RouteComponent() {
    const data = Route.useLoaderData();
    const [isImporting, setIsImporting] = createSignal(data().isImporting);
    const [initalAnime, setInitalAnime] = createStore([...data().anime]);
    const [animes, setAnime] = createStore([...data().anime]);
    const animeIds = () => animes.map((an) => an.id);

    type ListSseEvent = {
        anime: Anime;
    };
    useSseStream<ListSseEvent>({
        url: "/api/v1/user/list/sse",
        onMessage: ({ anime }) => {
            const isInList = initalAnime.find((a) => a.id === anime.id);
            if (isInList) {
                console.log("anime already in list, skipping");
                return;
            }
            setInitalAnime((prev) => [...prev, anime]);
            setAnime((prev) => [...prev, anime]);
        },
    });

    const hasReordered = () => {
        return !animes.every((animeA, index) => initalAnime[index].id === animeA.id);
    };

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

        const updatedItems = animes.slice();
        updatedItems.splice(droppingAnime, 0, ...updatedItems.splice(draggingAnime, 1));
        setAnime(updatedItems);
    };

    const updateListOrderMutation = useMutation(() => ({
        mutationKey: ["list", "update"],
        mutationFn: async (ids: number[]) => {
            await updateListOrder(ids);
            setInitalAnime(ids.map((id) => animes.find((a) => a.id === id)!));
            toast("List order saved successfully");
        },
    }));

    const updateAnimeStatus = useMutation(() => ({
        mutationKey: ["anime", "status", "update"],
        mutationFn: async ({ animeId, status }: { animeId: number; status: AnimeListStatus }) => {},
    }));

    return (
        <fieldset disabled={updateListOrderMutation.isPending || updateAnimeStatus.isPending}>
            <BackToTop />
            <HeaderButtonsPortal>
                <Motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.25 }}>
                    <div class="flex gap-2">
                        <Button disabled={!hasReordered()} onClick={() => updateListOrderMutation.mutate(animeIds())}>
                            Save List Order
                        </Button>
                        <ResetButton hasReordered={hasReordered} reset={() => setAnime([...initalAnime])} />
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
                                Some animes in this list are still being importer. They should automatically appear once
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
                    <SortableProvider ids={animeIds()}>
                        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-3">
                            <For each={animes}>
                                {(anime, index) => (
                                    <DraggableAnimeCard
                                        index={index()}
                                        anime={anime}
                                        bringToFront={() => setAnime(moveIndexToStart(animes, index()))}
                                        setStatus={(status) => updateAnimeStatus.mutate({ animeId: anime.id, status })}
                                    />
                                )}
                            </For>
                        </div>
                    </SortableProvider>
                    <DragOverlay class={"transition-transform"}>
                        {(draggable) => <AnimeCard anime={animes.find((a) => a.id === draggable?.id)!} />}
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
