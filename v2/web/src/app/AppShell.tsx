import type { ReactNode } from "react";

import "./layout.css";

interface AppShellProps {
  statusLine?: ReactNode;
  protocolBanner: ReactNode;
  startupPanel: ReactNode;
  runtimeControls: ReactNode;
  paintToolbar: ReactNode;
  viewport: ReactNode;
  inspector: ReactNode;
  runHealth: ReactNode;
}

export function AppShell(props: AppShellProps) {
  return (
    <main className="app-shell">
      <header className="app-header">
        <h1>Petri V2 Control Surface</h1>
        <div className="app-status">{props.statusLine ?? null}</div>
        <div className="app-banner">
          <strong>Protocol</strong>
          <span>{props.protocolBanner}</span>
        </div>
      </header>

      <section className="app-grid">
        <aside className="rail left-rail">
          <section className="panel">
            <h2>Startup Draft</h2>
            {props.startupPanel}
          </section>
          <section className="panel">
            <h2>Runtime Controls</h2>
            {props.runtimeControls}
          </section>
          <section className="panel">
            <h2>Paint</h2>
            {props.paintToolbar}
          </section>
        </aside>

        <section className="panel viewport-panel">
          <h2>Viewport</h2>
          {props.viewport}
        </section>

        <aside className="rail right-rail">
          <section className="panel">
            <h2>Inspector</h2>
            {props.inspector}
          </section>
          <section className="panel">
            <h2>Run Health</h2>
            {props.runHealth}
          </section>
        </aside>
      </section>
    </main>
  );
}
