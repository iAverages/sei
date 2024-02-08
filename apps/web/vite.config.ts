import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
import devtools from "solid-devtools/vite";

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
    server: {
        port: 3000,
    },
    build: {
        target: "esnext",
    },
});
