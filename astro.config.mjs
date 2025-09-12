// @ts-check
import { defineConfig } from "astro/config";
import tsconfigPaths from "vite-tsconfig-paths";

import vue from "@astrojs/vue";

// https://astro.build/config
export default defineConfig({
  publicDir: "./desktop-app/frontend/public",
  srcDir: "./desktop-app/frontend/src",
  outDir: "./.cismu/dist",
  cacheDir: "./.cismu/.astro",
  vite: {
    plugins: [tsconfigPaths()],
  },

  integrations: [vue()],
});
