import { AnimeList, RelatedAnime } from "~/hooks/useAnimeList";

export const isStatus = (anime: AnimeList | RelatedAnime, status: string[]) => {
  return status.includes(anime.status);
};

export const isWatchStatus = (anime: AnimeList, status: string[]) => {
  return status.includes(anime.watch_status);
};
