import { z } from "zod/v4";
import { api } from "./fetch";
import { trytm } from "./utils";

export const listSchema = z.object({
    animes: z.array(
        z.object({
            english_title: z.string().nullish(),
            id: z.number(),
            picture: z.string(),
            romaji_title: z.string(),
            season: z.string().nullish(),
            season_year: z.number().nullish(),
            status: z.string(),
        }),
    ),
    list_entries: z.array(
        z.object({
            anime_id: z.number(),
            created_at: z.string(),
            status: z.string(),
            updated_at: z.string(),
            user_id: z.string(),
            watch_priority: z.number(),
        }),
    ),
    relations: z.array(z.object({ anime_id: z.number(), relation_id: z.number() })),
    isImporting: z.boolean(),
});

const buildSeriesMap = (animeId: number, relations: z.infer<typeof listSchema>["relations"], map: number[]) => {
    let updatedMap = [...map];
    for (const rel of relations) {
        if (map.includes(rel.anime_id)) continue;
        if (rel.relation_id === animeId) {
            updatedMap = buildSeriesMap(rel.anime_id, relations, [rel.anime_id, ...map]);
        }
    }

    for (const rel of relations) {
        if (map.includes(rel.relation_id)) continue;
        if (rel.anime_id === animeId)
            updatedMap = buildSeriesMap(rel.relation_id, relations, [...updatedMap, rel.relation_id]);
    }

    return updatedMap;
};

export const fetchList = async () => {
    const data = await api("/api/v1/user/list");
    const [json, error] = await trytm(data.json());
    if (error) throw new Error("failed to fetch list");

    const validator = listSchema.safeParse(json);
    if (!validator.success) throw validator.error;

    const orderedAnime = [];
    const seriesMap = [];

    for (const anime of validator.data.animes) {
        const listStatus = validator.data.list_entries.find((status) => status.anime_id === anime.id);
        if (!listStatus) {
            // console.warn("failed to find list status for anime", anime.id);
            continue;
        }

        if (listStatus.watch_priority === 0) orderedAnime.push(anime);
        else orderedAnime[listStatus.watch_priority] = anime;

        seriesMap.push(buildSeriesMap(anime.id, validator.data.relations, [anime.id]));
    }

    return {
        isImporting: validator.data.isImporting,
        anime: orderedAnime.filter((i) => !!i),
        seriesMap,
    };
};

export type Anime = Awaited<ReturnType<typeof fetchList>>["anime"][number];

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
};

export const AnimeListStatus = {
    Watching: "Watching",
    Complete: "Complete",
    OnHold: "OnHold",
    Dropped: "Dropped",
    PlanToWatch: "PlanToWatch",
} as const;
export type AnimeListStatus = keyof typeof AnimeListStatus;
