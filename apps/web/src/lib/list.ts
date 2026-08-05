import { z } from "zod/v4";
import { api } from "./fetch";

export const listVisibilitySchema = z.enum(["PRIVATE", "UNLISTED", "PUBLIC"]);

export const listSummarySchema = z
    .object({
        id: z.string(),
        name: z.string(),
        visibility: listVisibilitySchema,
        isDefault: z.boolean(),
    })
    .strict();

export const animeSchema = z
    .object({
        id: z.number().int(),
        englishTitle: z.string().nullable(),
        romajiTitle: z.string(),
        status: z.string(),
        picture: z.string().nullable(),
        season: z.string().nullable(),
        seasonYear: z.number().int().nullable(),
    })
    .strict();

export const listDetailSchema = z
    .object({
        list: listSummarySchema,
        anime: z.array(animeSchema),
    })
    .strict();

const listSummariesSchema = z.array(listSummarySchema);
const animeSearchSchema = z.array(animeSchema);
const importStatusSchema = z.object({ isImporting: z.boolean() }).strict();
const apiErrorSchema = z.object({ message: z.string() }).passthrough();

export type ListVisibility = z.infer<typeof listVisibilitySchema>;
export type ListSummary = z.infer<typeof listSummarySchema>;
export type ListDetail = z.infer<typeof listDetailSchema>;
export type Anime = z.infer<typeof animeSchema>;

export class ListApiError extends Error {
    constructor(
        message: string,
        readonly status: number,
    ) {
        super(message);
    }
}

const responseError = async (response: Response, action: string) => {
    let message: string | undefined;
    try {
        const parsed = apiErrorSchema.safeParse(await response.json());
        if (parsed.success) message = parsed.data.message;
    } catch {
        // The status still provides useful context for non-JSON errors.
    }
    return new ListApiError(message ?? `${action} (${response.status} ${response.statusText})`, response.status);
};

const requestJson = async <T>(response: Response, schema: z.ZodType<T>, action: string): Promise<T> => {
    if (!response.ok) throw await responseError(response, action);
    return schema.parse(await response.json());
};

const jsonRequest = (method: string, body: unknown): RequestInit => ({
    method,
    body: JSON.stringify(body),
    headers: { "Content-Type": "application/json" },
});

export const fetchLists = async () =>
    requestJson(await api("/api/v1/lists"), listSummariesSchema, "Failed to fetch lists");

export const fetchList = async (listId: string) =>
    requestJson(await api(`/api/v1/lists/${encodeURIComponent(listId)}`), listDetailSchema, "Failed to fetch list");

export const createList = async (input: { name: string; visibility: ListVisibility; animeIds: number[] }) =>
    requestJson(await api("/api/v1/lists", jsonRequest("POST", input)), listDetailSchema, "Failed to create list");

export const updateList = async (listId: string, input: { name: string; visibility: ListVisibility }) =>
    requestJson(
        await api(`/api/v1/lists/${encodeURIComponent(listId)}`, jsonRequest("PATCH", input)),
        listDetailSchema,
        "Failed to update list",
    );

export const addListEntries = async (listId: string, animeIds: number[]) =>
    requestJson(
        await api(`/api/v1/lists/${encodeURIComponent(listId)}/entries`, jsonRequest("POST", { animeIds })),
        listDetailSchema,
        "Failed to add anime to list",
    );

export const removeListEntry = async (listId: string, animeId: number) => {
    const response = await api(`/api/v1/lists/${encodeURIComponent(listId)}/entries/${encodeURIComponent(animeId)}`, {
        method: "DELETE",
    });
    if (!response.ok) throw await responseError(response, "Failed to remove anime from list");
};

export const updateListOrder = async (listId: string, ids: number[]) => {
    const response = await api(`/api/v1/lists/${encodeURIComponent(listId)}/order`, jsonRequest("PUT", { ids }));
    if (!response.ok) throw await responseError(response, "Failed to save list order");
};

export const searchAnime = async (query: string) => {
    const trimmedQuery = query.trim().slice(0, 30);
    if (trimmedQuery.length < 2) return [];
    const search = new URLSearchParams({ q: trimmedQuery });
    return requestJson(await api(`/api/v1/anime/search?${search}`), animeSearchSchema, "Failed to search anime");
};

export const fetchPublicList = async (listId: string) =>
    requestJson(
        await api(`/api/v1/public/lists/${encodeURIComponent(listId)}`),
        listDetailSchema,
        "Failed to fetch shared list",
    );

export const fetchImportStatus = async () =>
    requestJson(await api("/api/v1/user/import-status"), importStatusSchema, "Failed to fetch import status").then(
        ({ isImporting }) => isImporting,
    );
