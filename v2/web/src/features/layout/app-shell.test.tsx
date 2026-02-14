import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { AppShell } from "../../app/AppShell";
import { ProtocolBanner } from "../protocol/ProtocolBanner";

describe("AppShell", () => {
  it("renders required stage-4 surface regions", () => {
    const html = renderToStaticMarkup(
      <AppShell
        startupPanel={<div>startup-panel</div>}
        runtimeControls={<div>runtime-controls</div>}
        paintToolbar={<div>paint-toolbar</div>}
        viewport={<div>viewport</div>}
        inspector={<div>inspector</div>}
        runHealth={<div>run-health</div>}
        protocolBanner={<div>protocol-banner</div>}
      />
    );

    expect(html).toContain("Startup Draft");
    expect(html).toContain("Runtime Controls");
    expect(html).toContain("Paint");
    expect(html).toContain("Viewport");
    expect(html).toContain("Inspector");
    expect(html).toContain("Run Health");
    expect(html).toContain("Protocol");
  });

  it("keeps critical desktop control labels visible", () => {
    const html = renderToStaticMarkup(
      <AppShell
        startupPanel={<button type="button">Apply Startup</button>}
        runtimeControls={
          <div>
            <button type="button">Start</button>
            <button type="button">Pause</button>
            <button type="button">Step</button>
          </div>
        }
        paintToolbar={
          <div>
            <button type="button">Food Tool</button>
            <button type="button">Clear All</button>
          </div>
        }
        viewport={<div data-testid="viewport-canvas">viewport</div>}
        inspector={<div>Selected Creature</div>}
        runHealth={<div>Population</div>}
        protocolBanner={<div>v2alpha1 (compatible) | connected</div>}
      />
    );

    expect(html).toContain("Apply Startup");
    expect(html).toContain("Start");
    expect(html).toContain("Pause");
    expect(html).toContain("Step");
    expect(html).toContain("Food Tool");
    expect(html).toContain("Clear All");
    expect(html).toContain("Protocol");
  });
});

describe("ProtocolBanner", () => {
  it("renders explicit connecting/reconnecting/error states", () => {
    const connecting = renderToStaticMarkup(
      <ProtocolBanner
        protocolVersion="v2alpha1"
        protocolMismatch={false}
        connectionState="connecting"
        errorMessage={null}
      />
    );
    expect(connecting).toContain("connecting");

    const reconnecting = renderToStaticMarkup(
      <ProtocolBanner
        protocolVersion="v2alpha1"
        protocolMismatch={false}
        connectionState="reconnecting"
        errorMessage={null}
      />
    );
    expect(reconnecting).toContain("reconnecting");

    const error = renderToStaticMarkup(
      <ProtocolBanner
        protocolVersion="v2alpha1"
        protocolMismatch={false}
        connectionState="error"
        errorMessage="startup failed"
      />
    );
    expect(error).toContain("error");
    expect(error).toContain("startup failed");
  });

  it("prioritizes protocol_mismatch state over connection state", () => {
    const html = renderToStaticMarkup(
      <ProtocolBanner
        protocolVersion="v2alpha1"
        protocolMismatch
        connectionState="connected"
        errorMessage={null}
      />
    );

    expect(html).toContain("protocol_mismatch");
    expect(html).toContain("version mismatch");
  });
});
