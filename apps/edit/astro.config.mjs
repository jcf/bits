import { defineConfig } from "astro/config";

const site = process.env.ASTRO_SITE ?? "https://usebits.app";

export default defineConfig({
  site,
  vite: {
    ssr: {
      noExternal: ["@bits/shared"],
    },
    server: {
      allowedHosts: ["edit.invetica.dev"],
      hmr: {
        clientPort: 443,
        protocol: "wss",
      },
    },
  },
});
