import { describe, expect, it } from 'vitest';
import { parseSseMessages } from './sse';

describe('parseSseMessages', () => {
  it('parses a single SSE event block', () => {
    const { messages, remainder } = parseSseMessages(
      'event: started\ndata: {"type":"started","scan_id":"scan-1"}\n\n',
    );

    expect(messages).toEqual([
      {
        event: 'started',
        data: '{"type":"started","scan_id":"scan-1"}',
      },
    ]);
    expect(remainder).toBe('');
  });

  it('keeps an incomplete trailing block in the remainder', () => {
    const { messages, remainder } = parseSseMessages(
      'event: progress\ndata: {"type":"progress"}\n\nevent: complete\ndata: {"type":"complete"}\n',
    );

    expect(messages).toHaveLength(1);
    expect(messages[0]?.event).toBe('progress');
    expect(remainder).toBe('event: complete\ndata: {"type":"complete"}\n');
  });
});
