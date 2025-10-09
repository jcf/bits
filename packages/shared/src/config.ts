import { DOMAIN_EDIT, DOMAIN_PAGE, DOMAIN_WWW } from "astro:env/client";

const config = {
  apps: {
    edit: {
      domain: DOMAIN_EDIT,
      url: `https://${DOMAIN_EDIT}/`,
    },
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
