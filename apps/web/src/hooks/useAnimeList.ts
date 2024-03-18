import { createQuery } from "@tanstack/solid-query";
import { Accessor } from "solid-js";

export const AnimeReleaseStatus = {
    Finished: "FINISHED",
    NotYetReleased: "NOT_YET_RELEASED",
    Releasing: "RELEASING",
} as const;

export type AnimeReleaseStatus = (typeof AnimeReleaseStatus)[keyof typeof AnimeReleaseStatus];

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

// The status of the users list
// importing: The list is currently being imported
// updating: The list is currently being updated
// imported: The list has been imported
export const UserListStatus = {
    Importing: "importing",
    Updating: "updating",
    Imported: "imported",
} as const;

export type UserListStatus = (typeof UserListStatus)[keyof typeof UserListStatus];

export const ListStatus = {
    Watching: "WATCHING",
    Completed: "COMPLETED",
    PlanToWatch: "PLAN_TO_WATCH",
    Dropped: "DROPPED",
    OnHold: "ON_HOLD",
} as const;

export type ListStatus = (typeof ListStatus)[keyof typeof ListStatus];

export type ListStatusItem = {
    anime_id: number;
    status: ListStatus;
    watch_priority: number;
};

export const useAnimeList = ({ hasReordered }: { hasReordered: Accessor<boolean> }) => {
    return createQuery(() => ({
        enabled: !hasReordered(),
        staleTime: 1000 * 60 * 5,
        queryKey: ["anime", "list"],
        queryFn: async () => {
            const res = await fetch(`${import.meta.env.PUBLIC_API_URL ?? ""}/api/v1/anime`, {
                credentials: "include",
            });

            if (!res.ok) {
                throw res;
            }
            // TODO: Do proper validation
            const anime = (await res.json()) as {
                animes: Anime[];
                list_status: ListStatusItem[];
                import_status: UserListStatus;
                relations: {
                    anime_id: number;
                    relation: "SEQUEL" | "PREQUEL";
                    related_id: number;
                }[];
            };

            return anime;
        },
    }));
};
