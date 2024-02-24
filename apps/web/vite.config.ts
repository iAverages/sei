import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
import devtools from "solid-devtools/vite";

// Ensure this runs on build to verify envs are set on build
// import "~/env";

export default defineConfig({
    plugins: [
        devtools({
            autoname: true, // e.g. enable autoname
            locator: {
                targetIDE: "vscode",
            },
        }),
        solidPlugin(),
        tsconfigPaths(),
    ],
    envDir: "../..",
    server: {
        port: 3000,
    },
    build: {
        target: "esnext",
    },
});
