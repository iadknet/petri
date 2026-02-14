import type {
  FramePayload,
  StatusPayload,
  WsEventEnvelope,
} from "../../protocol/models";

export type SimulationSnapshot = {
  status: StatusPayload | null;
  frame: FramePayload | null;
};

export function asErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export function applyWsEventSnapshot(
  snapshot: SimulationSnapshot,
  event: WsEventEnvelope
): SimulationSnapshot {
  if (event.event === "status") {
    return {
      status: event.payload,
      frame: snapshot.frame,
    };
  }

  if (event.event === "frame") {
    return {
      status: snapshot.status,
      frame: event.payload,
    };
  }

  return snapshot;
}

export function selectNextCreatureId(
  frame: FramePayload | null,
  selectedCreatureId: number | null
): number | null {
  if (!frame || frame.creatures.length === 0) {
    return null;
  }

  if (selectedCreatureId === null) {
    return frame.creatures[0].id;
  }

  if (frame.creatures.some((creature) => creature.id === selectedCreatureId)) {
    return selectedCreatureId;
  }

  return frame.creatures[0].id;
}
