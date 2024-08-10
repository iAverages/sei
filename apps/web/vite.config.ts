import { Plugin, PluginOption, defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
import devtools from "solid-devtools/vite";
import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";
import path from "path";

// Ensure this runs on build to verify envs are set on build
// import "./src/env";

const a = (): PluginOption => {
  return {
    name: "env",
    async config(config, envConfig) {
      const { normalizePath, loadEnv } = await import("vite");
      const rootDir = userConfig.root || cwd();

      const resolvedRoot = normalizePath("./");

      const envDir = userConfig.envDir
        ? normalizePath(path.resolve(resolvedRoot, userConfig.envDir))
        : resolvedRoot;

      const env = loadEnv(envConfig.mode, envDir, userConfig.envPrefix);
      return {
        define: {
          "import.meta.env.PUBLIC_API_URL": JSON.stringify(
            process.env.PUBLIC_API_URL,
          ),
        },
      };
    },
  };
};

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
  envPrefix: "PUBLIC_",
});
