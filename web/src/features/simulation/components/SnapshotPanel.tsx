import { useState } from "react";

import { WorldSnapshot } from "../../../protocol";

type SnapshotPanelProps = {
  onExportSnapshot: () => Promise<string>;
  onImportSnapshot: (snapshot: WorldSnapshot) => Promise<void>;
};

export function SnapshotPanel({ onExportSnapshot, onImportSnapshot }: SnapshotPanelProps) {
  const [rawSnapshot, setRawSnapshot] = useState("");
  const [message, setMessage] = useState<string | null>(null);

  return (
    <section className="section">
      <h2>Snapshot</h2>
      <div className="button-row">
        <button
          className="primary-btn"
          onClick={async () => {
            try {
              const next = await onExportSnapshot();
              setRawSnapshot(next);
              setMessage("Snapshot exported.");
            } catch (error) {
              setMessage((error as Error).message || "Failed to export snapshot.");
            }
          }}
        >
          Export Snapshot
        </button>
        <button
          className="primary-btn"
          onClick={async () => {
            try {
              const parsed = JSON.parse(rawSnapshot) as WorldSnapshot;
              await onImportSnapshot(parsed);
              setMessage("Snapshot imported.");
            } catch (error) {
              setMessage((error as Error).message || "Failed to import snapshot.");
            }
          }}
        >
          Import Snapshot
        </button>
      </div>

      <textarea
        aria-label="Snapshot JSON"
        value={rawSnapshot}
        onChange={(event) => setRawSnapshot(event.target.value)}
        placeholder="Snapshot JSON"
        rows={8}
        style={{ width: "100%", resize: "vertical" }}
      />

      {message ? <p className="muted">{message}</p> : null}
    </section>
  );
}
