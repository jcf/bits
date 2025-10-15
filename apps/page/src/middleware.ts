import { defineMiddleware } from "astro:middleware";

export const onRequest = defineMiddleware(async (context, next) => {
  const hostname = new URL(context.request.url).hostname;

  // Extract subdomain: foo.localhost or foo.page.dev -> foo
  const subdomain = hostname.split(".")[0];

  // Simple tenant data
  const tenants = {
    jcf: { name: "James Conroy-Finn" },
  };

  // Check if this looks like a tenant request (not www, localhost, or base domain)
  const isTenantRequest =
    subdomain &&
    subdomain !== "localhost" &&
    subdomain !== "page" &&
    subdomain !== "www";

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
