import { defineConfig } from "astro/config";
import vercel from "@astrojs/vercel";

const site = process.env.ASTRO_SITE ?? "https://bits.page";

export default defineConfig({
  site,
  output: 'server', // SSR mode for dynamic tenant loading
  adapter: vercel(),
  vite: {
    ssr: {
      noExternal: ["@bits/shared"],
    },
    server: {
      allowedHosts: [".page.invetica.dev", "page.invetica.dev"],
      hmr: {
        clientPort: 443,
        protocol: "wss",
      },
    },
  },
});
