import { useDragDropContext } from "@thisbeyond/solid-dnd";
import {
  DragDropProvider,
  DragDropSensors,
  DragOverlay,
  SortableProvider,
  closestCenter,
} from "@thisbeyond/solid-dnd";
import {
  For,
  Show,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Button } from "~/components/ui/button";
import {
  AnimeCardInnerContent,
  DraggableAnimeCard,
} from "~/components/anime-card";
import { createUpdateListOrder } from "~/hooks/createUpdateListOrder";
import { useBeforeLeave } from "@solidjs/router";
import { Anime, useAnimeList } from "~/hooks/useAnimeList";

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

export default function Home() {
  const [hasReordered, setHasReordered] = createSignal(false);
  const userList = useAnimeList({ hasReordered });
  const updateListOrder = createUpdateListOrder();

  const [items, setItems] = createSignal(userList.data?.animes);

  createEffect(() => setItems(userList.data?.animes ?? []));
  const ids = () => items().map((item) => item.id);

  const getEntry = (anime: Anime) => {
    return userList.data.list_entries.find(
      (entry) => entry.anime_id === anime.id,
    );
  };

  const onDragEnd = ({ draggable, droppable }) => {
    if (draggable && droppable) {
      const currentItems = items();
      const fromIndex = currentItems.findIndex((a) => a.id === draggable.id);
      const toIndex = currentItems.findIndex((a) => a.id === droppable.id);
      if (fromIndex !== toIndex) {
        const updatedItems = currentItems.slice();
        updatedItems.splice(toIndex, 0, ...updatedItems.splice(fromIndex, 1));
        setItems(updatedItems);
      }

      setHasReordered(
        items().some((anime, index) => {
          const userListItem = getEntry(anime);
          if (userListItem?.watch_priority !== index + 1) {
            return true;
          }
        }),
      );
    }
  };

  useBeforeLeave((e) => {
    if (updateListOrder.isPending) {
      e.preventDefault();
      if (
        window.confirm(
          "You have unsaved changes, are you sure you want to leave?",
        )
      ) {
        e.retry(true);
      }
      return;
    }
    if (hasReordered()) {
      e.preventDefault();
      if (
        window.confirm(
          "You have unsaved changes, are you sure you want to leave?",
        )
      ) {
        e.retry(true);
      }
    }
  });

  return (
    <div class={"p-6 flex flex-col gap-3"}>
      <Button
        onClick={() => updateListOrder.mutate(items()?.map((i) => i.id))}
        class={"bg-blue-500"}
      >
        Update List Order
      </Button>
      <Show when={userList.data}>
        <DragDropProvider
          onDragEnd={onDragEnd}
          collisionDetector={closestCenter}
        >
          <Fix /> {/* See definition */}
          <DragDropSensors />
          <SortableProvider ids={ids()}>
            <div class={"grid grid-cols-2 md:grid-cols-3 lg:grid-cols-8 gap-3"}>
              <For each={items()}>
                {(anime) => (
                  <DraggableAnimeCard
                    anime={anime}
                    bringToFront={() => {}}
                    disabled={userList.isLoading}
                  />
                )}
              </For>
            </div>
          </SortableProvider>
          <DragOverlay class={"transition-transform"}>
            {(draggable) => (
              <AnimeCardInnerContent
                anime={items().find((a) => a.id === draggable.id)}
              />
            )}
          </DragOverlay>
        </DragDropProvider>
      </Show>
    </div>
  );
}
