import { useDragDropContext } from "@thisbeyond/solid-dnd";
import { createSortable } from "@thisbeyond/solid-dnd";
import { Show, createSignal } from "solid-js";
import { Anime, ListStatus, ListStatusItem, UserListStatus } from "~/hooks/useAnimeList";

import { Badge } from "~/components/ui/badge";
import { cn } from "~/lib/utils";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuGroup,
    ContextMenuItem,
    ContextMenuPortal,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from "~/components/ui/context-menu";
import { FaSolidArrowUpRightFromSquare } from "solid-icons/fa";
import { isStatus } from "~/lib/status";

type AnimeCardProps = {
    grouped?: boolean;
    anime: Anime;
    disabled?: boolean;
    showOverlayInfo?: boolean;
    listStatus?: ListStatusItem;
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
            <Show when={props.listStatus.status === ListStatus.OnHold}>
                <Note class={"bg-yellow-300 hover:bg-yellow-300"}>On Hold</Note>
            </Show>
            <Show when={props.listStatus.status === ListStatus.Dropped}>
                <Note class={"bg-red-400 hover:bg-red-400"}>Dropped</Note>
            </Show>

            {/* <Show
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
            </Show> */}
            <Show when={!props.listStatus}>
                <Note class={"bg-yellow-300 hover:bg-yellow-300"}>Not In List</Note>
            </Show>
        </div>
    );
};

const AnimeCardInnerContent = (props: AnimeCardProps) => {
    return (
        <div
            class={"flex transition-transform"}
            classList={{
                "pointer-events-none": props.disabled,
            }}>
            <div
                class={"transition-transform flex flex-col items-center text-center "}
                classList={{
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

export const AnimeCardInner = (props: AnimeCardProps) => {
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
    const [watchStatus, setWatchStatus] = createSignal(props.listStatus?.status);

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
                    {/* <ContextMenuSub overlap>
                        <ContextMenuSubTrigger>GitHub</ContextMenuSubTrigger>
                        <ContextMenuPortal>
                                <ContextMenuItem>Create Pull Request…</ContextMenuItem>
                                <ContextMenuItem>View Pull Requests</ContextMenuItem>
                                <ContextMenuItem>Sync Fork</ContextMenuItem>
                                <ContextMenuSeparator />
                                <ContextMenuItem>Open on GitHub</ContextMenuItem>
                            </ContextMenuSubContent>
                        </ContextMenuPortal>
                    </ContextMenuSub> */}
                    <ContextMenuSub overlap>
                        <ContextMenuSubTrigger>Watch Status</ContextMenuSubTrigger>
                        <ContextMenuPortal>
                            <ContextMenuSubContent>
                                <ContextMenuGroup>
                                    <ContextMenuRadioGroup
                                        value={watchStatus()}
                                        onChange={(state) => {
                                            // setWatchStatus(state);
                                        }}>
                                        <ContextMenuRadioItem value="WATCHING">Watching</ContextMenuRadioItem>
                                        <ContextMenuRadioItem value="COMPLETED">Completed</ContextMenuRadioItem>
                                        <ContextMenuRadioItem value="PLAN_TO_WATCH">Plan To Watch</ContextMenuRadioItem>
                                        <ContextMenuRadioItem value="DROPPED">Dropped</ContextMenuRadioItem>
                                    </ContextMenuRadioGroup>
                                </ContextMenuGroup>
                            </ContextMenuSubContent>
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

export const AnimeCard = (props: AnimeCardProps & { bringToFront: () => void }) => {
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
