import { defineConfig } from "astro/config";

const site = process.env.ASTRO_SITE ?? "https://edit.usebits.app";

const allowedHosts = [process.env.DOMAIN_EDIT];

export default defineConfig({
  site,
  vite: {
    ssr: {
      noExternal: ["@bits/shared"],
    },
    server: {
      allowedHosts,
      strictPort: true,
      hmr: {
        clientPort: 443,
        protocol: "wss",
      },
    },
  },
});
