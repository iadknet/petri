import { CreatureDetail, CreatureSnapshot } from "../../../protocol";

type CreatureInspectorPanelProps = {
  creature: CreatureSnapshot | null;
  detail: CreatureDetail | null;
};

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

export function CreatureInspectorPanel({ creature, detail }: CreatureInspectorPanelProps) {
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
          {detail ? (
            <>
              <div className="metric-row">
                <span>Move blocked (last tick)</span>
                <strong>{detail.last_move_blocked ? "Yes" : "No"}</strong>
              </div>

              <h3>Inputs</h3>
              <div className="metric-row">
                <span>Food direction</span>
                <strong>{formatEnergy(detail.last_inputs.food_direction)}</strong>
              </div>
              <div className="metric-row">
                <span>Food distance</span>
                <strong>{formatEnergy(detail.last_inputs.food_distance)}</strong>
              </div>
              <div className="metric-row">
                <span>Creature direction</span>
                <strong>{formatEnergy(detail.last_inputs.creature_direction)}</strong>
              </div>
              <div className="metric-row">
                <span>Creature distance</span>
                <strong>{formatEnergy(detail.last_inputs.creature_distance)}</strong>
              </div>
              <div className="metric-row">
                <span>Barrier direction</span>
                <strong>{formatEnergy(detail.last_inputs.barrier_direction)}</strong>
              </div>
              <div className="metric-row">
                <span>Barrier distance</span>
                <strong>{formatEnergy(detail.last_inputs.barrier_distance)}</strong>
              </div>
              <div className="metric-row">
                <span>Local density</span>
                <strong>{formatEnergy(detail.last_inputs.local_density)}</strong>
              </div>
              <div className="metric-row">
                <span>Move blocked input</span>
                <strong>{formatEnergy(detail.last_inputs.move_blocked_last_tick)}</strong>
              </div>

              <h3>Outputs</h3>
              <div className="metric-row">
                <span>Move X</span>
                <strong>{formatEnergy(detail.last_outputs.move_x)}</strong>
              </div>
              <div className="metric-row">
                <span>Move Y</span>
                <strong>{formatEnergy(detail.last_outputs.move_y)}</strong>
              </div>
              <div className="metric-row">
                <span>Eat</span>
                <strong>{formatEnergy(detail.last_outputs.eat)}</strong>
              </div>
              <div className="metric-row">
                <span>Reproduce</span>
                <strong>{formatEnergy(detail.last_outputs.reproduce)}</strong>
              </div>
            </>
          ) : (
            <p className="muted">Loading per-creature details...</p>
          )}
        </>
      ) : (
        <p className="muted">Click a creature in the viewport to inspect it.</p>
      )}
    </section>
  );
}
