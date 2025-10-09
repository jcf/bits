import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

const site = process.env.ASTRO_SITE ?? "https://www.usebits.app";
const allowedHosts = [process.env.DOMAIN_WWW];

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

    plugins: [tailwindcss()],
  },
});
