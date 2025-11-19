import type { Config } from "@/utils/config";
import defaultConfig from "@/utils/config";

export function subURL(sub: string): URL;
export function subURL(config: Config, sub: string): URL;
export function subURL(configOrSub: Config | string, maybeSub?: string): URL {
  const config = typeof configOrSub === "string" ? defaultConfig : configOrSub;
  const sub = typeof configOrSub === "string" ? configOrSub : maybeSub!;
  return new URL(`https://${sub}.${config.apps.page.domain}/`);
}
