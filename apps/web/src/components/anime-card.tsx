import { useDragDropContext } from "@thisbeyond/solid-dnd";
import { createSortable } from "@thisbeyond/solid-dnd";
import { createSignal } from "solid-js";
import { Anime, } from "~/hooks/useAnimeList";

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

type AnimeCardProps = {
    disabled?: boolean;
    anime: Anime;
};

export const AnimeCardInnerContent = (props: AnimeCardProps) => {
    return (
        <div
            class={"flex transition-transform"}
            classList={{
                "pointer-events-none opacity-60": props.disabled,
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

export const DraggableAnimeCard = (props: AnimeCardProps & { bringToFront: () => void }) => {
    const sortable = createSortable(props.anime.id);
    const [state] = useDragDropContext();
    const [watchStatus, _setWatchStatus] = createSignal("WATCHING");

    return (
        <div
            use:sortable
            class="sortable transition-opacity touch-none"
            classList={{
                "opacity-25 duration-250": sortable.isActiveDraggable || props.disabled,
                "transition-transform": !!state.active.draggable,
            }}>
            <div class={"flex relative items-center justify-center w-full"}>
                <ContextMenu>
                    <ContextMenuTrigger>
                        <AnimeCardInnerContent {...props} />
                    </ContextMenuTrigger>
                    <ContextMenuContent class="w-48">
                        <ContextMenuItem onClick={props.bringToFront}>
                            <span>Bring To Front</span>
                        </ContextMenuItem>

                        <ContextMenuSub overlap>
                            <ContextMenuSubTrigger>Watch Status</ContextMenuSubTrigger>
                            <ContextMenuPortal>
                                <ContextMenuSubContent>
                                    <ContextMenuGroup>
                                        <ContextMenuRadioGroup
                                            value={watchStatus()}
                                            onChange={(_state) => {
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
        </div>
    );
};
