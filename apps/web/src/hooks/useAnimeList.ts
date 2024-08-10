import { createQuery } from "@tanstack/solid-query";
import { Accessor } from "solid-js";

export const AnimeReleaseStatus = {
  Finished: "FINISHED",
  NotYetReleased: "NOT_YET_RELEASED",
  Releasing: "RELEASING",
} as const;

export type AnimeReleaseStatus =
  (typeof AnimeReleaseStatus)[keyof typeof AnimeReleaseStatus];

export const AnimeWatchStatus = {
  Watching: "watching",
  Completed: "completed",
  OnHold: "on_hold",
  Dropped: "dropped",
  PlanToWatch: "plan_to_watch",
};

export type AnimeWatchStatus =
  (typeof AnimeWatchStatus)[keyof typeof AnimeWatchStatus];

export type Anime = {
  id: number;
  romaji_title: string;
  status: AnimeReleaseStatus;
  picture: string;
  season?: string;
  season_year?: number;
  created_at: string;
  updated_at: string;
};

export type ListEntry = {
  anime_id: number;
  watch_priority: number;
  watch_status: AnimeWatchStatus;
};

export const useAnimeList = ({
  hasReordered,
}: {
  hasReordered: Accessor<boolean>;
}) => {
  return createQuery(() => ({
    enabled: !hasReordered(),
    staleTime: 1000 * 60 * 5,
    queryKey: ["anime", "list"],
    queryFn: async () => {
      const res = await fetch(
        `${import.meta.env.PUBLIC_API_URL ?? ""}/api/v1/user/list`,
        {
          credentials: "include",
        },
      );

      if (!res.ok) {
        throw res;
      }
      // TODO: Do proper validation
      const data = (await res.json()) as {
        animes: Anime[];
        list_entries: ListEntry[];
      };

      data.animes = data.animes.sort((a, b) => {
        const aEntry = data.list_entries.find(
          (entry) => entry.anime_id === a.id,
        )!;
        const bEntry = data.list_entries.find(
          (entry) => entry.anime_id === b.id,
        )!;

        const ap = aEntry.watch_priority;
        const bp = bEntry.watch_priority;

        // Ensure newly added entries are shown last rather than first;
        // Happens when a new anime is added to the list
        // or changes from releasing to complete
        if (ap == 0) return -1;
        if (bp == 0) return -1;

        if (ap < bp) return -1;
        if (ap > bp) return 1;

        return 0;
      });

      return data;
    },
  }));
};
