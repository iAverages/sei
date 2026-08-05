import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/solid-start/plugin/vite";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import viteSolid from "vite-plugin-solid";
import tsConfigPaths from "vite-tsconfig-paths";

export default defineConfig({
    server: {
        port: 3000,
    },
    plugins: [
        tsConfigPaths({
            projects: ["./tsconfig.json"],
        }),
        devtools(),
        tailwindcss(),
        tanstackStart(),
        viteSolid({ ssr: true }),
        nitro({
            devServer: { runner: "self" },
            routeRules: {
                "/api/**": {
                    proxy: {
                        to: "http://localhost:3001/api/**",
                        fetchOptions: { redirect: "manual" },
                    },
                },
            },
        }),
    ],
});
