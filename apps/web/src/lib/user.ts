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

export type User = z.infer<typeof userSchema>;
