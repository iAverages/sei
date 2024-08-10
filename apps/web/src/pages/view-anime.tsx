import { A, useParams } from "@solidjs/router";
import { For, Show, createMemo } from "solid-js";
import { Status, useAnime } from "~/hooks/useAnime";

const Login = () => {
  const params = useParams();
  const animes = useAnime({ animeId: () => params.id });

  const animeData = createMemo(() => {
    const a = animes.data?.animes.map((anime) => ({
      ...anime,
      list_status: animes.data?.list_status.find(
        (status) => status.anime_id == anime.id,
      ) as Status | undefined,
    }));

    console.log("AnimeData", a);
    return a;
  });

  return (
    <div class={"w-screen h-screen items-center justify-center"}>
      <h1 class={"text-3xl font-bold"}>Anime</h1>
      <div class={"flex gap-5"}>
        <For each={animeData()}>
          {(anime) => (
            <A href={`/anime/${anime.id}`}>
              <div
                class={"flex flex-col items-center gap-1"}
                classList={{
                  "scale-110 bg-blue-600":
                    anime.id == (params.id as unknown as number),
                  "bg-green-300": anime.list_status?.status == "WATCHING",
                  "bg-yellow-300": anime.list_status?.status == "COMPLETED",
                  "bg-red-300": anime.list_status === undefined,
                }}
              >
                <img src={anime.picture} alt={anime.romaji_title} />
                <h2>{anime.romaji_title}</h2>
                <Show when={anime.list_status?.status}>
                  <span>{anime.list_status.status}</span>
                </Show>
              </div>
            </A>
          )}
        </For>
      </div>
    </div>
  );
};

export default Login;
