import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { AppShell } from "../../app/AppShell";

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
});
