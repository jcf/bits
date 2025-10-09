import { defineConfig, envField } from "astro/config";
import vercel from "@astrojs/vercel";
import tailwindcss from "@tailwindcss/vite";

const site = process.env.ASTRO_SITE ?? "https://bits.page";
const pageDomain = process.env.DOMAIN_PAGE;
const allowedHosts = [pageDomain, `.${pageDomain}`];

export default defineConfig({
  site,
  output: "server", // SSR mode for dynamic tenant loading
  adapter: vercel(),
  env: {
    schema: {
      DOMAIN_EDIT: envField.string({ context: "client", access: "public" }),
      DOMAIN_PAGE: envField.string({ context: "client", access: "public" }),
      DOMAIN_WWW: envField.string({ context: "client", access: "public" }),
    },
  },
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
