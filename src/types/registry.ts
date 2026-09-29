export type ProxyConfig = {
  httpProxy?: string;
  httpsProxy?: string;
  noProxy?: string;
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
  httpProxy: string;
  httpsProxy: string;
  noProxy: string;
  username: string;
  password: string;
};
