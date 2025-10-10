import { z } from "zod/v4";
import { api } from "./fetch";
import { trytm } from "./utils";

export const listSchema = z.object({
    animes: z.array(
        z.object({
            created_at: z.string(),
            id: z.number(),
            picture: z.string(),
            romaji_title: z.string(),
            season: z.string().nullable(),
            season_year: z.number().nullable(),
            status: z.string(),
            updated_at: z.string(),
        }),
    ),
    list_entries: z.array(
        z.object({
            anime_id: z.number(),
            watch_priority: z.number(),
            watch_status: z.string(),
        }),
    ),
});

export const fetchList = async () => {
    const data = await api("/api/v1/user/list");
    const [json, error] = await trytm(data.json());
    if (error) throw new Error("failed to fetch list");

    const validator = listSchema.safeParse(json);
    if (!validator.success) throw validator.error;

    const orderedAnime = []; // Array.from({ length: validator.data.list_entries.length }, () => 0);

    for (const anime of validator.data.animes) {
        const listStatus = validator.data.list_entries.find((status) => status.anime_id === anime.id);
        if (!listStatus) {
            console.warn("failed to find list status for anime", anime.id);
            continue;
        }

        console.log({ anime: anime.id, listStatus });
        if (listStatus.watch_priority === 0) orderedAnime.push(anime);
        else orderedAnime[listStatus.watch_priority] = anime;
    }

    return orderedAnime.filter((i) => !!i);
};

export type Anime = Awaited<ReturnType<typeof fetchList>>[number];

export const updateListOrder = async (ids: number[]) => {
    const response = await api("/api/v1/user/list", {
        method: "POST",
        body: JSON.stringify({ ids }),
        headers: {
            "Content-Type": "application/json",
        },
    });

    if (response.status !== 201) {
        console.warn("failed save list, recieved non 200 status code", {
            response,
            json: await trytm(response.json()),
        });
        throw new Error("failed save list, recieved non 200 status code");
    }

    return response.json();
};
