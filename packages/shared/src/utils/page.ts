import type { Config } from "@bits/shared/config";
import defaultConfig from "@bits/shared/config";

export function subURL(sub: string): URL;
export function subURL(config: Config, sub: string): URL;
export function subURL(configOrSub: Config | string, maybeSub?: string): URL {
  const config = typeof configOrSub === "string" ? defaultConfig : configOrSub;
  const sub = typeof configOrSub === "string" ? configOrSub : maybeSub!;
  return new URL(`https://${sub}.${config.apps.page.domain}/`);
}
