export type ProxyConfig = {
  url?: string;
};

export type RegistryAuth = {
  username?: string;
  password?: string;
};

export type SearchImageResult = {
  fullName: string;
  description: string;
  stars: number;
  pulls: number;
  isOfficial: boolean;
};

export type PullImageResult = {
  imageRef: string;
  tarPath: string;
  layerCount: number;
};

export type PersistedConfig = {
  outputDir: string;
  defaultTag: string;
  platformOs: string;
  platformArchitecture: string;
  proxy: string;
  username: string;
  password: string;
};
