import { createQuery } from "@tanstack/solid-query";

export type BaseAnime = {
    created_at: string;
    english_title: string;
    id: number;
    picture: string;
    relation: RelatedAnime[];
    romaji_title: string;
    status: string;
    updated_at: string;
};

export type WatchSatus = string;

export type AnimeList = BaseAnime & {
    watch_status: WatchSatus;
    watch_priority: number;
};

export interface Anime {
    created_at: string;
    english_title?: string;
    id: number;
    picture: string;
    relation: RelatedAnime[];
    romaji_title: string;
    status: string;
    updated_at: string;
}

export type RelatedAnime = Anime & {
    base_anime_id: number;
    related_anime_id: number;
    relation: string;
};

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
            const res = await fetch(`${import.meta.env.PUBLIC_API_URL ?? ""}/api/v1/anime`, {
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
