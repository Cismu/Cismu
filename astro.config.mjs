// @ts-check
import { defineConfig } from 'astro/config';
import tsconfigPaths from 'vite-tsconfig-paths';

// https://astro.build/config
export default defineConfig({
    publicDir: './desktop-app/frontend/public',
    srcDir: './desktop-app/frontend/src',
    vite: {
        plugins: [tsconfigPaths()],
    },
});
