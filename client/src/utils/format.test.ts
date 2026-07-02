import { describe, expect, it } from 'vitest';
import { formatBytes, formatExtension } from './format';

describe('formatBytes', () => {
  it('formats gigabytes', () => {
    expect(formatBytes(4_589_934_592)).toBe('4.27 GB');
  });
});

describe('formatExtension', () => {
  it('uppercases extensions', () => {
    expect(formatExtension('mkv')).toBe('MKV');
  });
});
