import type { FrameCreature } from "../protocol/models";

interface CreatureInspectorProps {
  creature: FrameCreature | null;
}

export function CreatureInspector(props: CreatureInspectorProps) {
  if (!props.creature) {
    return <p className="muted">No creature selected.</p>;
  }

  const creature = props.creature;
  return (
    <div className="control-stack metric-list">
      <p className="metric-row">
        <span>ID</span>
        <strong>{creature.id}</strong>
      </p>
      <p className="metric-row">
        <span>Position</span>
        <strong>
          ({creature.x}, {creature.y})
        </strong>
      </p>
      <p className="metric-row">
        <span>Energy</span>
        <strong>{creature.energy.toFixed(2)}</strong>
      </p>
      <p className="metric-row">
        <span>Phenotype RGB</span>
        <strong>[{creature.phenotype_rgb.join(", ")}]</strong>
      </p>
    </div>
  );
}
