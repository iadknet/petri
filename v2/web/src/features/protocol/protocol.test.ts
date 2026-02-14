import { describe, expect, it } from "vitest";
import frameFixture from "../../fixtures/protocol-v2alpha1/frame.json";
import statusFixture from "../../fixtures/protocol-v2alpha1/status.json";
import wsFrameFixture from "../../fixtures/protocol-v2alpha1/ws-frame-event.json";
import wsMismatchedFixture from "../../fixtures/protocol-v2alpha1/ws-mismatched-event.json";
import wsStatusFixture from "../../fixtures/protocol-v2alpha1/ws-status-event.json";
import {
  decodeFramePayload,
  decodeStatusPayload,
  decodeWsEventEnvelope,
} from "./decoders";

describe("v2alpha1 protocol decoders", () => {
  it("decodes status and frame fixtures", () => {
    const status = decodeStatusPayload(statusFixture);
    const frame = decodeFramePayload(frameFixture);
    expect(status.protocol_version).toBe("v2alpha1");
    expect(frame.protocol_version).toBe("v2alpha1");
    expect(frame.width).toBeGreaterThan(0);
  });

  it("accepts matching websocket event payloads", () => {
    const statusEvent = decodeWsEventEnvelope(wsStatusFixture);
    const frameEvent = decodeWsEventEnvelope(wsFrameFixture);
    expect(statusEvent.event).toBe("status");
    expect(frameEvent.event).toBe("frame");
  });

  it("rejects event payload mismatches", () => {
    expect(() => decodeWsEventEnvelope(wsMismatchedFixture)).toThrow(
      /event\/payload mismatch/i
    );
  });
});
