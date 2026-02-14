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
    <div className="control-stack">
      <p>ID: {creature.id}</p>
      <p>
        Position: ({creature.x}, {creature.y})
      </p>
      <p>Energy: {creature.energy.toFixed(2)}</p>
      <p>
        Phenotype RGB: [{creature.phenotype_rgb.join(", ")}]
      </p>
    </div>
  );
}
