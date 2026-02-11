import { WorldFrame, decodeFrame } from "../../../protocol";

type FrameListener = (frame: WorldFrame) => void;
type ConnectionListener = (connected: boolean) => void;
type ErrorListener = (message: string) => void;

export class FrameStreamClient {
  private socket: WebSocket | null = null;

  constructor(private readonly wsUrl: string) {}

  connect(
    onFrame: FrameListener,
    onConnection: ConnectionListener,
    onError: ErrorListener
  ): () => void {
    this.socket = new WebSocket(this.wsUrl);
    this.socket.binaryType = "arraybuffer";

    this.socket.onopen = () => {
      onConnection(true);
    };
    this.socket.onclose = () => {
      onConnection(false);
    };
    this.socket.onerror = () => {
      onError("WebSocket error. Is petri-server running on port 4000?");
    };
    this.socket.onmessage = (event) => {
      try {
        if (event.data instanceof ArrayBuffer) {
          onFrame(decodeFrame(event.data));
        }
      } catch (decodeError) {
        onError(`Failed to decode frame: ${(decodeError as Error).message}`);
      }
    };

    return () => this.disconnect();
  }

  disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }
}
