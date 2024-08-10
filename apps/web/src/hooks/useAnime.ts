import { createQuery } from "@tanstack/solid-query";
import { Accessor } from "solid-js";

export interface AnimeData {
  animes: Anime[];
  list_status: Status[];
  series_animes: SeriesAnime[];
}

export interface Anime {
  created_at: string;
  english_title: any;
  id: number;
  picture: string;
  romaji_title: string;
  season: string;
  season_year: number;
  status: string;
  updated_at: string;
}

export interface Status {
  anime_id: number;
  status: string;
}

export interface SeriesAnime {
  anime_id: number;
  series_id: number;
  series_order: number;
}

export const useAnime = (props: { animeId: Accessor<string> }) => {
  return createQuery(() => ({
    enabled: !!props.animeId(),
    staleTime: 1000 * 60 * 5,
    queryKey: ["anime", props.animeId()],
    queryFn: async () => {
      console.log("Fetching", props.animeId());
      const res = await fetch(
        `${import.meta.env.PUBLIC_API_URL ?? ""}/api/v1/anime/${props.animeId()}`,
        {
          credentials: "include",
        },
      );

      if (!res.ok) {
        throw res;
      }

      return (await res.json()) as AnimeData;
    },
  }));
};
