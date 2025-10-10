import { createSortable, useDragDropContext } from "@thisbeyond/solid-dnd";
import type { Anime } from "~/lib/list";

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

export const DraggableAnimeCard = (props: { anime: Anime; disabled?: boolean }) => {
    const sortable = createSortable(props.anime.id);
    const [state] = useDragDropContext()!;

    return (
        <div
            use:sortable
            class="sortable transition-opacity touch-none"
            classList={{
                "opacity-25 duration-250": sortable.isActiveDraggable || props.disabled,
                "transition-transform": !!state.active.draggable,
            }}
        >
            <AnimeCard anime={props.anime} />
        </div>
    );
};
