import { defineMiddleware } from "astro:middleware";

const domain = process.env.DOMAIN_PAGE;

function extractSubdomain(hostname, domain) {
  if (!hostname.endsWith(`.${domain}`)) return null;
  return hostname.slice(0, -(domain.length + 1));
}

export const onRequest = defineMiddleware(async (context, next) => {
  const hostname = new URL(context.request.url).hostname;

  const subdomain = extractSubdomain(hostname, domain);

  // Simple tenant data
  const tenants = {
    jcf: { name: "James Conroy-Finn" },
  };

  const isTenantRequest =
    subdomain !== null &&
    subdomain !== "localhost" &&
    subdomain !== "page" &&
    subdomain !== "www";

  console.log("Routing bits.page request", { subdomain, isTenantRequest });

  if (isTenantRequest) {
    if (tenants[subdomain]) {
      // Valid tenant
      context.locals.tenant = tenants[subdomain];
      context.locals.subdomain = subdomain;
    } else {
      // Invalid tenant - set flag for pages to handle
      context.locals.invalidTenant = true;
      context.locals.subdomain = subdomain;
    }
  }

  return next();
});
