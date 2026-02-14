import { useMemo } from "react";
import frameFixture from "./fixtures/protocol-v2alpha1/frame.json";
import statusFixture from "./fixtures/protocol-v2alpha1/status.json";
import {
  decodeFramePayload,
  decodeStatusPayload,
} from "./features/protocol/decoders";

export default function App() {
  const status = useMemo(() => decodeStatusPayload(statusFixture), []);
  const frame = useMemo(() => decodeFramePayload(frameFixture), []);

  return (
    <main style={{ fontFamily: "Menlo, Monaco, monospace", padding: "1.5rem" }}>
      <h1>Petri V2 CP-3 Surface</h1>
      <p>
        protocol <strong>{status.protocol_version}</strong> | state{" "}
        <strong>{status.state}</strong> | tick <strong>{status.tick}</strong>
      </p>
      <section>
        <h2>Run Health</h2>
        <p>population: {status.population}</p>
        <p>mean energy: {status.mean_energy.toFixed(2)}</p>
        <p>
          births/deaths window: {status.births_last_window}/{status.deaths_last_window}
        </p>
      </section>
      <section>
        <h2>Viewport</h2>
        <p>
          world: {frame.width}x{frame.height}
        </p>
        <p>creatures: {frame.creatures.length}</p>
        <p>food cells: {frame.food.length}</p>
        <p>barriers: {frame.barriers.length}</p>
      </section>
      <section>
        <h2>Protocol Banner</h2>
        <p>ready (fixture-backed decode baseline)</p>
      </section>
    </main>
  );
}
