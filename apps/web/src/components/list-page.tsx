import { DragDropProvider, type DragDropProviderProps, DragOverlay } from "@dnd-kit/solid";
import { isSortable } from "@dnd-kit/solid/sortable";
import { useMutation } from "@tanstack/solid-query";
import { useBlocker, useRouter } from "@tanstack/solid-router";
import { type Accessor, createSignal, For, Show } from "solid-js";
import { toast } from "solid-sonner";
import { AnimeCard, DraggableAnimeCard } from "~/components/anime-card";
import { BackToTop } from "~/components/back-to-top";
import { ListForm } from "~/components/list-form";
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
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "~/components/ui/sheet";
import { HeaderButtonsPortal } from "~/components/ui/sidebar/buttons-portal";
import { type Anime, addListEntries, type ListDetail, removeListEntry, updateList, updateListOrder } from "~/lib/list";
import { moveIndexToStart } from "~/lib/utils";

export function ListPage(props: { detail: ListDetail }) {
    const router = useRouter();
    const listId = props.detail.list.id;
    const [anime, setAnime] = createSignal([...props.detail.anime]);
    const [baseline, setBaseline] = createSignal([...props.detail.anime]);
    const [editOpen, setEditOpen] = createSignal(false);
    const hasReordered = () => !anime().every((item, index) => baseline()[index]?.id === item.id);
    const blocker = useBlocker({
        shouldBlockFn: () => hasReordered(),
        enableBeforeUnload: () => hasReordered(),
        withResolver: true,
    });

    const orderMutation = useMutation(() => ({
        mutationKey: ["list", listId, "order"],
        mutationFn: async (ids: number[]) => updateListOrder(listId, ids),
        onSuccess: () => {
            setBaseline([...anime()]);
            toast("List order saved");
        },
        onError: (error) => toast.error(error.message),
    }));
    const removeMutation = useMutation(() => ({
        mutationKey: ["list", listId, "entry", "remove"],
        mutationFn: async (animeId: number) => removeListEntry(listId, animeId),
        onSuccess: async () => {
            toast("Anime removed from list");
            await router.invalidate();
        },
        onError: (error) => toast.error(error.message),
    }));

    const updateAnimeOrder = (items: Anime[]) => setAnime(items);
    const onDragEnd: NonNullable<DragDropProviderProps["onDragEnd"]> = ({ canceled, operation }) => {
        if (canceled || !isSortable(operation.source)) return;
        const sourceIndex = operation.source.initialIndex;
        const targetIndex = operation.source.index;
        if (sourceIndex === targetIndex) return;

        const updatedItems = anime().slice();
        updatedItems.splice(targetIndex, 0, ...updatedItems.splice(sourceIndex, 1));
        updateAnimeOrder(updatedItems);
    };
    const busy = () => orderMutation.isPending || removeMutation.isPending;

    return (
        <fieldset disabled={busy()} class="flex flex-col gap-3">
            <BackToTop />
            <HeaderButtonsPortal>
                <div class="ml-auto flex flex-wrap gap-2">
                    <Button variant="outline" disabled={hasReordered()} onClick={() => setEditOpen(true)}>
                        Edit
                    </Button>
                    <Button
                        disabled={!hasReordered()}
                        onClick={() => orderMutation.mutate(anime().map(({ id }) => id))}
                    >
                        Save order
                    </Button>
                    <ResetButton hasReordered={hasReordered} reset={() => updateAnimeOrder([...baseline()])} />
                </div>
            </HeaderButtonsPortal>

            <Show
                when={anime().length > 0}
                fallback={
                    <section class="flex min-h-80 flex-col items-center justify-center rounded-lg border border-dashed border-border px-6 text-center">
                        <h1 class="text-xl font-semibold">This list is empty</h1>
                        <p class="mt-2 max-w-md text-sm text-muted-foreground">
                            {props.detail.list.isDefault
                                ? "Anime from your imported watching and planning list will appear here."
                                : "Use Edit to search your local anime catalog and add the first title."}
                        </p>
                        <Show when={!props.detail.list.isDefault}>
                            <Button class="mt-5" onClick={() => setEditOpen(true)}>
                                Add anime
                            </Button>
                        </Show>
                    </section>
                }
            >
                <DragDropProvider onDragEnd={onDragEnd}>
                    <div class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8">
                        <For each={anime()}>
                            {(item, index) => (
                                <DraggableAnimeCard
                                    index={index()}
                                    anime={item}
                                    disabled={busy()}
                                    bringToFront={() => updateAnimeOrder(moveIndexToStart(anime(), index()))}
                                    remove={
                                        props.detail.list.isDefault || hasReordered()
                                            ? undefined
                                            : () => removeMutation.mutate(item.id)
                                    }
                                />
                            )}
                        </For>
                    </div>
                    <DragOverlay class="transition-transform">
                        {(source) => <AnimeCard anime={anime().find(({ id }) => id === source.id)!} />}
                    </DragOverlay>
                </DragDropProvider>
            </Show>

            <Sheet open={editOpen()} onOpenChange={setEditOpen}>
                <SheetContent>
                    <SheetHeader>
                        <SheetTitle>Edit {props.detail.list.name}</SheetTitle>
                        <SheetDescription>
                            {props.detail.list.isDefault
                                ? "Choose who can view your default list."
                                : "Update list details or add more anime."}
                        </SheetDescription>
                    </SheetHeader>
                    <Show when={editOpen()}>
                        <ListForm
                            initialName={props.detail.list.name}
                            initialVisibility={props.detail.list.visibility}
                            existingAnimeIds={props.detail.anime.map(({ id }) => id)}
                            shareUrl={`/lists/${props.detail.list.slug}`}
                            visibilityOnly={props.detail.list.isDefault}
                            submitLabel="Save changes"
                            onSubmit={async ({ name, visibility, animeIds }) => {
                                await updateList(listId, { name, visibility });
                                if (animeIds.length > 0) await addListEntries(listId, animeIds);
                                setEditOpen(false);
                                await router.invalidate();
                            }}
                        />
                    </Show>
                </SheetContent>
            </Sheet>

            <AlertDialog
                open={blocker().status === "blocked"}
                onOpenChange={(open) => {
                    if (!open) blocker().reset?.();
                }}
            >
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>Leave without saving?</AlertDialogTitle>
                        <AlertDialogDescription>
                            Your unsaved list order will be lost if you leave this page.
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <Button variant="outline" onClick={() => blocker().reset?.()}>
                            Stay
                        </Button>
                        <Button onClick={() => blocker().proceed?.()}>Leave</Button>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </fieldset>
    );
}

const ResetButton = (props: { hasReordered: Accessor<boolean>; reset: () => void }) => {
    const [open, setOpen] = createSignal(false);
    return (
        <AlertDialog open={open()} onOpenChange={setOpen}>
            <AlertDialogTrigger as={Button} variant="outline" disabled={!props.hasReordered()}>
                Reset
            </AlertDialogTrigger>
            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Reset your unsaved order?</AlertDialogTitle>
                    <AlertDialogDescription>
                        This restores the order from the last saved version.
                    </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>
                        Cancel
                    </Button>
                    <Button
                        onClick={() => {
                            props.reset();
                            setOpen(false);
                        }}
                    >
                        Reset order
                    </Button>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    );
};
