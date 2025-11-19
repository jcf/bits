import { DOMAIN_PAGE, DOMAIN_WWW } from "astro:env/client";

export interface App {
  domain: string;
  url: string;
}

export interface Config {
  apps: Record<string, App>;
}

const config = {
  apps: {
    page: {
      domain: DOMAIN_PAGE,
      url: `https://${DOMAIN_PAGE}/`,
    },
    www: {
      domain: DOMAIN_WWW,
      url: `https://${DOMAIN_WWW}/`,
    },
  },
};

export default config;
