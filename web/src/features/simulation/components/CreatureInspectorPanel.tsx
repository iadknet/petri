import { CreatureDetail, CreatureSnapshot, InventoryItem } from "../../../protocol";

type CreatureInspectorPanelProps = {
  creature: CreatureSnapshot | null;
  detail: CreatureDetail | null;
};

const TOUCH_LABELS = ["Self", "North", "East", "South", "West"] as const;

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

function formatSlot(item: InventoryItem | null): string {
  if (!item) {
    return "empty";
  }
  if (item.kind === "barrier") {
    return "barrier";
  }
  return `food (${item.value.toFixed(3)})`;
}

function formatColorHex([r, g, b]: [number, number, number]): string {
  return `#${[r, g, b].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
}

export function CreatureInspectorPanel({ creature, detail }: CreatureInspectorPanelProps) {
  const phenotypeColor: [number, number, number] =
    detail?.phenotype_color ?? creature?.phenotype_color ?? [255, 255, 255];
  const phenotypeHex = formatColorHex(phenotypeColor);

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
          <div className="metric-row">
            <span>Phenotype color</span>
            <strong>
              <span
                aria-label={`Phenotype color ${phenotypeHex}`}
                style={{
                  display: "inline-block",
                  width: "0.85rem",
                  height: "0.85rem",
                  backgroundColor: `rgb(${phenotypeColor.join(",")})`,
                  border: "1px solid rgba(255, 255, 255, 0.4)",
                  marginRight: "0.45rem",
                  verticalAlign: "middle"
                }}
              />
              {phenotypeHex.toUpperCase()}
            </strong>
          </div>
          {detail ? (
            <>
              <div className="metric-row">
                <span>Move blocked (last tick)</span>
                <strong>{detail.last_move_blocked ? "Yes" : "No"}</strong>
              </div>

              <h3>Inventory</h3>
              <div className="metric-row">
                <span>Slot capacity</span>
                <strong>{detail.slot_capacity}</strong>
              </div>
              {detail.slots.map((slot, index) => (
                <div className="metric-row" key={`slot-${index + 1}`}>
                  <span>{`Slot ${index + 1}`}</span>
                  <strong>{formatSlot(slot)}</strong>
                </div>
              ))}

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

              <h3>Touch Sensors</h3>
              {TOUCH_LABELS.map((label, index) => (
                <div className="metric-row" key={`touch-${label}`}>
                  <span>{label}</span>
                  <strong>
                    {`exists=${detail.last_inputs.touch_exists[index]?.toFixed(0) ?? "0"} food=${formatEnergy(
                      detail.last_inputs.touch_food_value[index] ?? 0
                    )} barrier=${detail.last_inputs.touch_has_barrier[index]?.toFixed(0) ?? "0"} occupied=${detail.last_inputs.touch_occupied[index]?.toFixed(0) ?? "0"}`}
                  </strong>
                </div>
              ))}

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
              <div className="metric-row">
                <span>Inventory pickup</span>
                <strong>{formatEnergy(detail.last_outputs.inventory_pickup)}</strong>
              </div>
              <div className="metric-row">
                <span>Inventory put</span>
                <strong>{formatEnergy(detail.last_outputs.inventory_put)}</strong>
              </div>
              <div className="metric-row">
                <span>Inventory slot select</span>
                <strong>{formatEnergy(detail.last_outputs.inventory_slot_select)}</strong>
              </div>
              <div className="metric-row">
                <span>Inventory direction select</span>
                <strong>{formatEnergy(detail.last_outputs.inventory_direction_select)}</strong>
              </div>

              <h3>Illegal Attempts</h3>
              {detail.illegal_attempts.length === 0 ? (
                <p className="muted">No recent illegal attempts.</p>
              ) : (
                detail.illegal_attempts.map((attempt, index) => (
                  <div className="metric-row" key={`illegal-${index}-${attempt.tick}`}>
                    <span>{`Tick ${attempt.tick}`}</span>
                    <strong>{`${attempt.action} (${attempt.reason})`}</strong>
                  </div>
                ))
              )}
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
