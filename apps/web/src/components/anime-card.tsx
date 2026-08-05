import { useDragOperation } from "@dnd-kit/solid";
import { useSortable } from "@dnd-kit/solid/sortable";
import { type Anime, AnimeListStatus } from "~/lib/list";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuPortal,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from "./ui/context-menu";

export const AnimeCard = (props: { anime: Anime }) => {
    return (
        <div class="rounded-md bg-sidebar-accent relative overflow-hidden flex-grow flex-shrink basis-auto">
            <div class="size-full">
                <img
                    src={props.anime.picture}
                    alt={`${props.anime.romaji_title} banner`}
                    class="size-full min-h-[317px] max-h-[317px] object-cover"
                    draggable={false}
                />
            </div>

            <div class="absolute bottom-0 text-center font-semibold mt-auto w-full">
                <div>
                    <div class="bg-gradient-to-t from-sidebar/60 to-transparent h-12 w-full" />
                    <div class="bg-sidebar/60 p-2 pt-0">
                        <p>{props.anime.romaji_title}</p>
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
    setStatus: (status: AnimeListStatus) => void;
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
                    <ContextMenuSeparator />
                    <ContextMenuSub overlap>
                        <ContextMenuSubTrigger>MAL Status</ContextMenuSubTrigger>
                        <ContextMenuPortal>
                            <ContextMenuSubContent>
                                <ContextMenuItem onSelect={() => props.setStatus(AnimeListStatus.Watching)}>
                                    Watching
                                </ContextMenuItem>
                                <ContextMenuItem onSelect={() => props.setStatus(AnimeListStatus.Complete)}>
                                    Complete
                                </ContextMenuItem>
                                <ContextMenuItem onSelect={() => props.setStatus(AnimeListStatus.OnHold)}>
                                    On-Hold
                                </ContextMenuItem>
                                <ContextMenuItem onSelect={() => props.setStatus(AnimeListStatus.Dropped)}>
                                    Dropped
                                </ContextMenuItem>
                                <ContextMenuItem onSelect={() => props.setStatus(AnimeListStatus.PlanToWatch)}>
                                    Plan to Watch
                                </ContextMenuItem>
                            </ContextMenuSubContent>
                        </ContextMenuPortal>
                    </ContextMenuSub>
                </ContextMenuContent>
            </ContextMenu>
        </div>
    );
};
