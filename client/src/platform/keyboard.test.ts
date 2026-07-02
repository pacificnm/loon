import { describe, expect, it } from 'vitest';
import {
  isAppBackKey,
  isWebOsBackKey,
  WEBOS_BACK_KEYCODE,
} from './keyboard';

function keyEvent(key: string, keyCode = 0): KeyboardEvent {
  return { key, keyCode } as KeyboardEvent;
}

describe('keyboard helpers', () => {
  it('does not treat Backspace as webOS back', () => {
    const event = keyEvent('Backspace', 8);
    expect(isWebOsBackKey(event)).toBe(false);
    expect(isAppBackKey(event)).toBe(false);
  });

  it('detects webOS back keycode 461', () => {
    const event = keyEvent('Unidentified', WEBOS_BACK_KEYCODE);
    expect(isWebOsBackKey(event)).toBe(true);
  });

  it('detects browser back keys', () => {
    expect(isWebOsBackKey(keyEvent('GoBack'))).toBe(true);
    expect(isWebOsBackKey(keyEvent('BrowserBack'))).toBe(true);
  });
});
