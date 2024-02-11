import { createQuery } from "@tanstack/solid-query";

export interface AnimeList {
    node: Anime;
    list_status: AnimeListStatus;
}

export interface Anime {
    id: number;
    title: string;
    main_picture: {
        medium: string;
        large: string;
    };
    status: string;
    num_episodes: number;
    broadcast?: {
        day_of_the_week: string;
        start_time: string;
    };
}

export interface AnimeListStatus {
    status: string;
    score: number;
    num_episodes_watched: number;
    is_rewatching: boolean;
    updated_at: string;
}

export const ListStatus = {
    Importing: "importing",
    Updating: "updating",
    Imported: "imported",
} as const;

export type ListStatus = (typeof ListStatus)[keyof typeof ListStatus];

export const useAnimeList = () => {
    return createQuery(() => ({
        queryKey: ["anime", "list"],
        queryFn: async () => {
            const res = await fetch("http://localhost:3001/api/v1/anime", {
                credentials: "include",
            });

            if (!res.ok) {
                throw res;
            }
            const anime = (await res.json()) as {
                animes: AnimeList[];
                status: ListStatus;
            };

            return anime;
        },
    }));
};
