import { CreatureSnapshot } from "../../../protocol";

type CreatureInspectorPanelProps = {
  creature: CreatureSnapshot | null;
};

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

export function CreatureInspectorPanel({ creature }: CreatureInspectorPanelProps) {
  return (
    <section className="section">
      <h2>Creature Inspector</h2>
      {creature ? (
        <>
          <div className="metric-row">
            <span>ID</span>
            <strong>{creature.id}</strong>
          </div>
          <div className="metric-row">
            <span>Energy</span>
            <strong>{formatEnergy(creature.energy)}</strong>
          </div>
          <div className="metric-row">
            <span>Age</span>
            <strong>{creature.age}</strong>
          </div>
          <div className="metric-row">
            <span>Generation</span>
            <strong>{creature.generation}</strong>
          </div>
          <div className="metric-row">
            <span>Node count</span>
            <strong>{creature.node_count}</strong>
          </div>
        </>
      ) : (
        <p className="muted">Click a creature in the viewport to inspect it.</p>
      )}
    </section>
  );
}
