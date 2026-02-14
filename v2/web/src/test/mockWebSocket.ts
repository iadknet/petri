export class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  static instances: MockWebSocket[] = [];

  readyState = MockWebSocket.CONNECTING;
  binaryType: BinaryType = "blob";

  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(_url: string) {
    MockWebSocket.instances.push(this);

    queueMicrotask(() => {
      if (this.readyState !== MockWebSocket.CONNECTING) {
        return;
      }
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event("open"));
    });
  }

  close(): void {
    if (this.readyState === MockWebSocket.CLOSED) {
      return;
    }
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent("close"));
  }

  send(_data: string | ArrayBufferLike | Blob | ArrayBufferView): void {}

  addEventListener(): void {}

  removeEventListener(): void {}

  dispatchEvent(): boolean {
    return true;
  }
}

export function resetMockWebSockets(): void {
  for (const socket of MockWebSocket.instances) {
    socket.close();
  }
  MockWebSocket.instances = [];
}

export function emitSocketError(index = 0): void {
  const socket = MockWebSocket.instances[index];
  if (!socket) {
    throw new Error(`mock websocket instance ${index} not found`);
  }
  socket.onerror?.(new Event("error"));
}

export function emitSocketMessage(payload: unknown, index = 0): void {
  const socket = MockWebSocket.instances[index];
  if (!socket) {
    throw new Error(`mock websocket instance ${index} not found`);
  }

  socket.onmessage?.(
    new MessageEvent("message", {
      data: typeof payload === "string" ? payload : JSON.stringify(payload),
    })
  );
}
