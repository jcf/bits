import { defineConfig, envField } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

import mdx from "@astrojs/mdx";

const site = process.env.ASTRO_SITE ?? "https://www.usebits.app";
const allowedHosts = [process.env.DOMAIN_WWW];

export default defineConfig({
  site,

  env: {
    schema: {
      DOMAIN_EDIT: envField.string({ context: "client", access: "public" }),
      DOMAIN_PAGE: envField.string({ context: "client", access: "public" }),
      DOMAIN_WWW: envField.string({ context: "client", access: "public" }),
    },
  },

  vite: {
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

  integrations: [mdx()],
});
