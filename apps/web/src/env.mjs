import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

export const env = {};
// export const env = createEnv({
//     clientPrefix: "VITE_",
//     client: {
//         VITE_API_URL: z.string(),
//     },
//     runtimeEnvStrict: {
//         VITE_API_URL: import.meta.import.meta.env.VITE_API_URL ?? "",
//     },
//     emptyStringAsUndefined: true,
// });
