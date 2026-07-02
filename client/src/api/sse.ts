export interface SseMessage {
  event: string;
  data: string;
}

/** Parse one or more SSE message blocks from a text buffer. */
export function parseSseMessages(chunk: string): { messages: SseMessage[]; remainder: string } {
  const messages: SseMessage[] = [];
  const blocks = chunk.split('\n\n');
  const remainder = blocks.pop() ?? '';

  for (const block of blocks) {
    if (!block.trim()) {
      continue;
    }

    let event = 'message';
    const dataLines: string[] = [];

    for (const line of block.split('\n')) {
      if (line.startsWith('event:')) {
        event = line.slice(6).trim();
      } else if (line.startsWith('data:')) {
        dataLines.push(line.slice(5).trimStart());
      }
    }

    if (dataLines.length > 0) {
      messages.push({ event, data: dataLines.join('\n') });
    }
  }

  return { messages, remainder };
}

/** Read an SSE response body and invoke the callback for each message. */
export async function readSseStream(
  response: Response,
  onMessage: (message: SseMessage) => void,
  signal?: AbortSignal,
): Promise<void> {
  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error('Response has no body');
  }

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    if (signal?.aborted) {
      await reader.cancel();
      return;
    }

    const { done, value } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const parsed = parseSseMessages(buffer);
    buffer = parsed.remainder;
    for (const message of parsed.messages) {
      onMessage(message);
    }
  }

  buffer += decoder.decode();
  if (buffer.trim()) {
    const parsed = parseSseMessages(`${buffer}\n\n`);
    for (const message of parsed.messages) {
      onMessage(message);
    }
  }
}
