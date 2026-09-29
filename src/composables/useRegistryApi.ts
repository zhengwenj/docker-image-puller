import { invoke } from "@tauri-apps/api/core";
import type { PersistedConfig, PullImageResult, ProxyConfig, RegistryAuth, SearchImageResult } from "../types/registry";

export async function searchImages(keyword: string, proxy?: ProxyConfig, limit = 30) {
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
  return invoke<PullImageResult>("pull_image_as_tar", { request });
}

export async function testRegistryAuth(request: {
  registry?: string;
  repository?: string;
  tag?: string;
  proxy?: ProxyConfig;
  auth?: RegistryAuth;
}) {
  return invoke<{ imageRef?: string; message: string }>("test_registry_auth", { request });
}

export async function loadPersistedConfig() {
  return invoke<PersistedConfig>("load_persisted_config");
}

export async function savePersistedConfig(config: PersistedConfig) {
  return invoke("save_persisted_config", { config });
}
