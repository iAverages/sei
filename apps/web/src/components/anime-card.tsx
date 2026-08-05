import { useDragOperation } from "@dnd-kit/solid";
import { useSortable } from "@dnd-kit/solid/sortable";
import { Show } from "solid-js";
import type { Anime } from "~/lib/list";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger,
} from "./ui/context-menu";

export const AnimeCard = (props: { anime: Anime }) => {
    return (
        <div class="rounded-md bg-sidebar-accent relative overflow-hidden flex-grow flex-shrink basis-auto">
            <div class="size-full">
                <Show
                    when={props.anime.picture}
                    fallback={
                        <div class="flex min-h-[317px] items-center justify-center bg-muted px-4 text-center text-sm text-muted-foreground">
                            Poster unavailable
                        </div>
                    }
                >
                    {(picture) => (
                        <img
                            src={picture()}
                            alt={`${props.anime.romajiTitle} poster`}
                            class="size-full min-h-[317px] max-h-[317px] object-cover"
                            draggable={false}
                        />
                    )}
                </Show>
            </div>

            <div class="absolute bottom-0 text-center font-semibold mt-auto w-full">
                <div>
                    <div class="bg-gradient-to-t from-sidebar/60 to-transparent h-12 w-full" />
                    <div class="bg-sidebar/60 p-2 pt-0">
                        <p>{props.anime.romajiTitle}</p>
                    </div>
                </div>
            </div>
        </div>
    );
};

export const DraggableAnimeCard = (props: {
    anime: Anime;
    disabled?: boolean;
    bringToFront: () => void;
    index: number;
    remove?: () => void;
}) => {
    const sortable = useSortable({
        get id() {
            return props.anime.id;
        },
        get index() {
            return props.index;
        },
        get disabled() {
            return props.disabled ?? false;
        },
    });
    const operation = useDragOperation();

    return (
        <div
            ref={sortable.ref}
            class="sortable transition-opacity"
            classList={{
                "opacity-25 duration-250": sortable.isDragSource() || props.disabled,
                "transition-transform": !!operation.source(),
            }}
        >
            <ContextMenu>
                <ContextMenuTrigger>
                    <AnimeCard anime={props.anime} />
                </ContextMenuTrigger>
                <ContextMenuContent>
                    <ContextMenuItem class="cursor-pointer" onClick={props.bringToFront} disabled={props.index === 0}>
                        Bring to Front
                    </ContextMenuItem>
                    <ContextMenuItem
                        as="a"
                        href={`http://myanimelist.net/anime/${props.anime.id}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="cursor-pointer"
                    >
                        View on MAL
                    </ContextMenuItem>
                    {props.remove && (
                        <>
                            <ContextMenuSeparator />
                            <ContextMenuItem class="cursor-pointer text-destructive" onSelect={props.remove}>
                                Remove from list
                            </ContextMenuItem>
                        </>
                    )}
                </ContextMenuContent>
            </ContextMenu>
        </div>
    );
};
