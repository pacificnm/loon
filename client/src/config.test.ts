import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  LOON_SERVER_URL_KEY,
  clearServerUrl,
  getServerUrlOrNull,
  normalizeServerUrl,
  setServerUrl,
} from './config';

function mockLocalStorage() {
  const store = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => {
      store.set(key, value);
    },
    removeItem: (key: string) => {
      store.delete(key);
    },
  });
}

describe('normalizeServerUrl', () => {
  it('accepts http and https URLs', () => {
    expect(normalizeServerUrl('http://192.168.88.10:3000')).toBe(
      'http://192.168.88.10:3000',
    );
    expect(normalizeServerUrl('https://loon.local/')).toBe('https://loon.local');
  });

  it('rejects empty and non-http URLs', () => {
    expect(normalizeServerUrl('')).toBeNull();
    expect(normalizeServerUrl('192.168.1.1:3000')).toBeNull();
  });
});

describe('server url storage', () => {
  beforeEach(() => {
    mockLocalStorage();
  });

  afterEach(() => {
    clearServerUrl();
    vi.unstubAllGlobals();
  });

  it('persists and reads from localStorage', () => {
    setServerUrl('http://192.168.88.10:3000/');
    expect(localStorage.getItem(LOON_SERVER_URL_KEY)).toBe('http://192.168.88.10:3000');
    expect(getServerUrlOrNull()).toBe('http://192.168.88.10:3000');
  });
});
