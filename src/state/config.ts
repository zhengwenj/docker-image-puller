import { reactive } from "vue";
import { loadPersistedConfig, savePersistedConfig } from "../composables/useRegistryApi";
import type { PersistedConfig } from "../types/registry";

export const appConfig = reactive<PersistedConfig>({
  outputDir: "",
  defaultTag: "latest",
  platformOs: "linux",
  platformArchitecture: "amd64",
  httpProxy: "",
  httpsProxy: "",
  noProxy: "",
  username: "",
  password: "",
});

let loaded = false;

export async function ensureConfigLoaded() {
  if (loaded) return;
  try {
    Object.assign(appConfig, await loadPersistedConfig());
  } catch {
    // 首次启动没有持久化配置时使用默认值
  }
  loaded = true;
}

export function proxyConfigOrUndefined() {
  const proxy = {
    httpProxy: appConfig.httpProxy.trim(),
    httpsProxy: appConfig.httpsProxy.trim(),
    noProxy: appConfig.noProxy.trim(),
  };
  return proxy.httpProxy || proxy.httpsProxy || proxy.noProxy ? proxy : undefined;
}

export function authConfigOrUndefined() {
  const auth = {
    username: appConfig.username.trim(),
    password: appConfig.password,
  };
  return auth.username || auth.password.trim() ? auth : undefined;
}

export async function persistConfig() {
  await savePersistedConfig({ ...appConfig });
}

export function dirname(path: string) {
  const normalized = path.replace(/\//g, "\\");
  const idx = normalized.lastIndexOf("\\");
  return idx > 0 ? normalized.slice(0, idx) : normalized;
}

export function formatTime(date: Date) {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

export function resolvePickedDirectory(file: File & { path?: string; webkitRelativePath?: string }) {
  if (file.path) return dirname(file.path);
  if (file.webkitRelativePath) return file.webkitRelativePath.split("/")[0] || "";
  return "";
}
