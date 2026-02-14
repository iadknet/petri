import {
  PROTOCOL_VERSION,
  type ActionCounts,
  type ErrorEnvelope,
  type FrameBarrier,
  type FrameCreature,
  type FrameFood,
  type FramePayload,
  type HealthPayload,
  type LifecycleState,
  type StatusPayload,
  type WsEventEnvelope,
} from "./models";

type JsonObject = Record<string, unknown>;

function asObject(input: unknown, label: string): JsonObject {
  if (typeof input !== "object" || input === null || Array.isArray(input)) {
    throw new Error(`${label} must be an object`);
  }
  return input as JsonObject;
}

function assertProtocolVersion(version: unknown, label: string): string {
  if (typeof version !== "string") {
    throw new Error(`${label}.protocol_version must be a string`);
  }
  if (version !== PROTOCOL_VERSION) {
    throw new Error(`protocol version mismatch: expected ${PROTOCOL_VERSION}, got ${version}`);
  }
  return version;
}

function asNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`${label} must be a number`);
  }
  return value;
}

function asString(value: unknown, label: string): string {
  if (typeof value !== "string") {
    throw new Error(`${label} must be a string`);
  }
  return value;
}

function asState(value: unknown, label: string): LifecycleState {
  const state = asString(value, label);
  if (state !== "idle" && state !== "running" && state !== "paused") {
    throw new Error(`${label} must be one of idle|running|paused`);
  }
  return state;
}

function decodeActionCounts(input: unknown): ActionCounts {
  const object = asObject(input, "last_action_counts");
  return {
    move: asNumber(object.move, "last_action_counts.move"),
    eat: asNumber(object.eat, "last_action_counts.eat"),
    reproduce: asNumber(object.reproduce, "last_action_counts.reproduce"),
    inventory_pickup: asNumber(
      object.inventory_pickup,
      "last_action_counts.inventory_pickup"
    ),
    inventory_put: asNumber(object.inventory_put, "last_action_counts.inventory_put"),
    noop: asNumber(object.noop, "last_action_counts.noop"),
  };
}

function decodeCreature(input: unknown): FrameCreature {
  const object = asObject(input, "creature");
  const phenotypeRaw = object.phenotype_rgb;
  if (!Array.isArray(phenotypeRaw) || phenotypeRaw.length !== 3) {
    throw new Error("creature.phenotype_rgb must be [r,g,b]");
  }
  return {
    id: asNumber(object.id, "creature.id"),
    x: asNumber(object.x, "creature.x"),
    y: asNumber(object.y, "creature.y"),
    energy: asNumber(object.energy, "creature.energy"),
    phenotype_rgb: [
      asNumber(phenotypeRaw[0], "creature.phenotype_rgb[0]"),
      asNumber(phenotypeRaw[1], "creature.phenotype_rgb[1]"),
      asNumber(phenotypeRaw[2], "creature.phenotype_rgb[2]"),
    ],
  };
}

function decodeFood(input: unknown): FrameFood {
  const object = asObject(input, "food");
  return {
    x: asNumber(object.x, "food.x"),
    y: asNumber(object.y, "food.y"),
    density: asNumber(object.density, "food.density"),
  };
}

function decodeBarrier(input: unknown): FrameBarrier {
  const object = asObject(input, "barrier");
  return {
    x: asNumber(object.x, "barrier.x"),
    y: asNumber(object.y, "barrier.y"),
  };
}

export function decodeStatusPayload(input: unknown): StatusPayload {
  const object = asObject(input, "status");
  return {
    protocol_version: assertProtocolVersion(object.protocol_version, "status"),
    state: asState(object.state, "status.state"),
    tick: asNumber(object.tick, "status.tick"),
    sensor_radius: asNumber(object.sensor_radius, "status.sensor_radius"),
    health_window_ticks: asNumber(
      object.health_window_ticks,
      "status.health_window_ticks"
    ),
    population: asNumber(object.population, "status.population"),
    mean_energy: asNumber(object.mean_energy, "status.mean_energy"),
    births_last_window: asNumber(
      object.births_last_window,
      "status.births_last_window"
    ),
    deaths_last_window: asNumber(
      object.deaths_last_window,
      "status.deaths_last_window"
    ),
    last_action_counts: decodeActionCounts(object.last_action_counts),
  };
}

export function decodeFramePayload(input: unknown): FramePayload {
  const object = asObject(input, "frame");
  const creaturesRaw = object.creatures;
  const foodRaw = object.food;
  const barriersRaw = object.barriers;
  if (!Array.isArray(creaturesRaw) || !Array.isArray(foodRaw) || !Array.isArray(barriersRaw)) {
    throw new Error("frame arrays must be arrays");
  }

  return {
    protocol_version: assertProtocolVersion(object.protocol_version, "frame"),
    tick: asNumber(object.tick, "frame.tick"),
    width: asNumber(object.width, "frame.width"),
    height: asNumber(object.height, "frame.height"),
    creatures: creaturesRaw.map(decodeCreature),
    food: foodRaw.map(decodeFood),
    barriers: barriersRaw.map(decodeBarrier),
  };
}

export function decodeHealthPayload(input: unknown): HealthPayload {
  const object = asObject(input, "health");
  return {
    population: asNumber(object.population, "health.population"),
    genome_node_count_p50: asNumber(
      object.genome_node_count_p50,
      "health.genome_node_count_p50"
    ),
    genome_node_count_p90: asNumber(
      object.genome_node_count_p90,
      "health.genome_node_count_p90"
    ),
    mean_energy: asNumber(object.mean_energy, "health.mean_energy"),
  };
}

export function decodeErrorEnvelope(input: unknown): ErrorEnvelope {
  const object = asObject(input, "error envelope");
  const error = asObject(object.error, "error envelope.error");
  return {
    protocol_version: assertProtocolVersion(object.protocol_version, "error envelope"),
    error: {
      code: asString(error.code, "error.code"),
      message: asString(error.message, "error.message"),
      details:
        typeof error.details === "undefined"
          ? undefined
          : (asObject(error.details, "error.details") as ErrorEnvelope["error"]["details"]),
    },
  };
}

export function decodeWsEventEnvelope(input: unknown): WsEventEnvelope {
  const object = asObject(input, "ws event");
  const protocol_version = assertProtocolVersion(object.protocol_version, "ws event");
  const event = asString(object.event, "ws event.event");
  const tick = asNumber(object.tick, "ws event.tick");

  try {
    if (event === "status") {
      const payload = decodeStatusPayload(object.payload);
      return { protocol_version, event, tick, payload };
    }
    if (event === "frame") {
      const payload = decodeFramePayload(object.payload);
      return { protocol_version, event, tick, payload };
    }
    if (event === "health") {
      const payload = decodeHealthPayload(object.payload);
      return { protocol_version, event, tick, payload };
    }
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`event/payload mismatch: ${reason}`);
  }

  throw new Error(`unknown ws event kind: ${event}`);
}
