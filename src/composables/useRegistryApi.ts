import { invoke } from "@tauri-apps/api/core";
import type { PersistedConfig, PullImageResult, ProxyConfig, RegistryAuth, SearchImageResult } from "../types/registry";

// 浏览器预览模式(非 Tauri 环境)下的 mock,便于脱离后端调试 UI;Tauri 内不受影响
const inBrowserPreview = typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window);

const mockSearchPool: SearchImageResult[] = [
  { fullName: "library/nginx", description: "Official build of Nginx, the high-performance web server and reverse proxy.", stars: 19800, pulls: 372000000, isOfficial: true },
  { fullName: "library/redis", description: "Redis is an open source key-value store that functions as a data structure server.", stars: 12500, pulls: 148000000, isOfficial: true },
  { fullName: "library/postgres", description: "The PostgreSQL Object Relational Database Management System.", stars: 11200, pulls: 121000000, isOfficial: true },
  { fullName: "bitnami/nginx", description: "Bitnami container image for Nginx, packaged for production deployments.", stars: 860, pulls: 32000000, isOfficial: false },
  { fullName: "library/node", description: "Node.js is a JavaScript-based platform for server-side and networking applications.", stars: 13900, pulls: 254000000, isOfficial: true },
  { fullName: "rancher/mirrored-library-traefik", description: "Traefik reverse proxy mirror maintained by Rancher for RKE2.", stars: 42, pulls: 8100000, isOfficial: false },
];

async function mockSearch(keyword: string): Promise<SearchImageResult[]> {
  await new Promise((r) => setTimeout(r, 500));
  const lower = keyword.toLowerCase();
  return mockSearchPool
    .filter((item) => item.fullName.toLowerCase().includes(lower) || item.description.toLowerCase().includes(lower))
    .slice(0, 8);
}

export async function searchImages(keyword: string, proxy?: ProxyConfig, limit = 30) {
  if (inBrowserPreview) return mockSearch(keyword);
  return invoke<SearchImageResult[]>("search_images", {
    request: { keyword, limit, proxy },
  });
}

export async function pullImageAsTar(request: {
  registry?: string;
  repository: string;
  tag?: string;
  outputDir: string;
  tarFileName?: string;
  platformOs?: string;
  platformArchitecture?: string;
  proxy?: ProxyConfig;
  auth?: RegistryAuth;
}) {
  if (inBrowserPreview) {
    await new Promise((r) => setTimeout(r, 1800));
    return {
      imageRef: `${request.registry || "docker.io"}/${request.repository}:${request.tag || "latest"}`,
      tarPath: `${request.outputDir}/${request.repository.replace(/\//g, "_")}_${request.tag || "latest"}.tar`,
      layerCount: 7,
    } satisfies PullImageResult;
  }
  return invoke<PullImageResult>("pull_image_as_tar", { request });
}

export async function testRegistryAuth(request: {
  registry?: string;
  repository?: string;
  tag?: string;
  proxy?: ProxyConfig;
  auth?: RegistryAuth;
}) {
  if (inBrowserPreview) {
    await new Promise((r) => setTimeout(r, 800));
    return { imageRef: `${request.registry || "docker.io"}/${request.repository || "library/nginx"}`, message: "连接正常,鉴权有效(预览模式)" };
  }
  return invoke<{ imageRef?: string; message: string }>("test_registry_auth", { request });
}

export async function loadPersistedConfig() {
  if (inBrowserPreview) {
    return {
      outputDir: "E:/docker-images",
      defaultTag: "latest",
      platformOs: "linux",
      platformArchitecture: "amd64",
      proxy: "",
      username: "",
      password: "",
    } satisfies PersistedConfig;
  }
  return invoke<PersistedConfig>("load_persisted_config");
}

export async function savePersistedConfig(config: PersistedConfig) {
  if (inBrowserPreview) return;
  return invoke("save_persisted_config", { config });
}
