/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_LOON_SERVER: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
