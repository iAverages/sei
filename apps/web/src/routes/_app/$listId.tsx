import { DragDropProvider, type DragDropProviderProps, DragOverlay } from "@dnd-kit/solid";
import { isSortable } from "@dnd-kit/solid/sortable";
import { useMutation } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import { type Accessor, batch, createSignal, For } from "solid-js";
import { createStore } from "solid-js/store";
import { toast } from "solid-sonner";
import { AnimeCard, DraggableAnimeCard } from "~/components/anime-card";
import { BackToTop } from "~/components/back-to-top";
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
import { type AnimeListStatus, fetchList, updateListOrder } from "~/lib/list";
import { moveIndexToStart } from "~/lib/utils";

export const Route = createFileRoute("/_app/$listId")({
    component: RouteComponent,
    ssr: false,
    loader: async () => {
        const { anime } = await fetchList();
        return {
            crumb: "Default",
            anime,
        };
    },
});
const useArrayStore = <TData, TId>(defaultValue: TData[], opts: { getId: (data: TData) => TId }) => {
    const getIds = (data: TData[]) => data.map(opts.getId);
    // only data is reactive, ids is used to checking if values exist in data but faster and
    // does not need to be reactive
    const [data, setData] = createStore(defaultValue);
    // ids is an array since we need the item ids as an array for the dnd components
    // and do not want to recreate it every time the data changes
    const [ids, setIds] = createStore(getIds(data));
    // however we want a fast lookup for animes we  have stored, we use a map which is
    // itemId -> index in storage for faster lookups
    const idsMap = new Map(ids.map((id, index) => [id, index]));

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
        data,
        remove,
        replace,
    };
};

function RouteComponent() {
    const data = Route.useLoaderData();
    const [hasReordered, setHasReordered] = createSignal(false);
    let initalAnime = [...data().anime];
    const {
        data: animes,
        ids: animeIdsArray,
        replace: setAnimes,
    } = useArrayStore([...data().anime], { getId: (data) => data.id });

    const updateAnimeOrder = (updatedItems: typeof animes) => {
        setAnimes(updatedItems);
        setHasReordered(!updatedItems.every((anime, index) => initalAnime[index]?.id === anime.id));
    };

    const onDragEnd: NonNullable<DragDropProviderProps["onDragEnd"]> = ({ canceled, operation }) => {
        if (canceled || !isSortable(operation.source)) return;
        const draggingAnime = operation.source.initialIndex;
        const droppingAnime = operation.source.index;
        if (draggingAnime === droppingAnime) return;

        const updatedItems = animes.slice();
        updatedItems.splice(droppingAnime, 0, ...updatedItems.splice(draggingAnime, 1));
        updateAnimeOrder(updatedItems);
    };

    const updateListOrderMutation = useMutation(() => ({
        mutationKey: ["list", "update"],
        mutationFn: async (ids: number[]) => {
            await updateListOrder(ids);
            initalAnime = ids.map((id) => animes.find((a) => a.id === id)!);
            setHasReordered(false);
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
                <div>
                    <div class="flex gap-2">
                        <Button
                            disabled={!hasReordered()}
                            onClick={() => updateListOrderMutation.mutate(animeIdsArray)}
                        >
                            Save List Order
                        </Button>
                        <ResetButton hasReordered={hasReordered} reset={() => updateAnimeOrder([...initalAnime])} />
                    </div>
                </div>
            </HeaderButtonsPortal>

            <div>
                <DragDropProvider onDragEnd={onDragEnd}>
                    <div class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8">
                        <For each={animes}>
                            {(anime, index) => (
                                <DraggableAnimeCard
                                    index={index()}
                                    anime={anime}
                                    disabled={updateListOrderMutation.isPending || updateAnimeStatus.isPending}
                                    bringToFront={() => updateAnimeOrder(moveIndexToStart(animes, index()))}
                                    setStatus={(status) => updateAnimeStatus.mutate({ animeId: anime.id, status })}
                                />
                            )}
                        </For>
                    </div>
                    <DragOverlay class={"transition-transform"}>
                        {(source) => {
                            return <AnimeCard anime={animes.find((anime) => anime.id === source.id)!} />;
                        }}
                    </DragOverlay>
                </DragDropProvider>
            </div>
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
