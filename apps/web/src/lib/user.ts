import z from "zod/v4";
import { api } from "./fetch";
import { trytm } from "./utils";

const userSchema = z.object({
    created_at: z.string(),
    id: z.string(),
    mal_id: z.number(),
    name: z.string(),
    picture: z.string(),
});

export const fetchUser = async () => {
    const response = await api("/api/v1/auth/me");
    const [json, error] = await trytm(response.json());
    if (error) return null;
    const validator = userSchema.safeParse(json);
    if (validator.success) return validator.data;
    return null;
};

export const logout = async () => {
    const response = await api("/api/v1/auth/logout", { method: "DELETE" });
    if (!response.ok) throw new Error("Failed to log out");
};

export const refreshMalAnime = async () => {
    const response = await api("/api/v1/user/list/refresh", { method: "POST" });
    const body = (await response.json().catch(() => null)) as { message?: string } | null;
    if (!response.ok) throw new Error(body?.message ?? "Failed to refresh MAL anime list");
};

export type User = z.infer<typeof userSchema>;
