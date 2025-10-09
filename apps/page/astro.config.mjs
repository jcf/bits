import { defineConfig } from "astro/config";
import vercel from "@astrojs/vercel";
import tailwindcss from "@tailwindcss/vite";

const site = process.env.ASTRO_SITE ?? "https://bits.page";
const pageDomain = process.env.DOMAIN_PAGE;
const allowedHosts = [pageDomain, `.${pageDomain}`];

export default defineConfig({
  site,
  output: "server", // SSR mode for dynamic tenant loading
  adapter: vercel(),
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

    plugins: [tailwindcss()],
  },
});
