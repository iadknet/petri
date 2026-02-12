import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import * as simulationStoreModule from "./features/simulation/store/simulationStore";
import { getMockStatus, setMockStatus } from "./test/handlers";
import { server } from "./test/server";

describe("App", () => {
  beforeEach(() => {
    setMockStatus({
      phase: "idle",
      run_id: null,
      pending_restart: false,
      startup_viable: true,
      startup_viability_code: null,
      startup_viability_message: null
    });
  });

  it("renders idle placeholder before starting", async () => {
    render(<App />);
    expect(await screen.findByText("Simulation not started")).toBeInTheDocument();
  });

  it("disables start when startup is non-viable", async () => {
    setMockStatus({
      startup_viable: false,
      startup_viability_code: "non_viable_startup_config",
      startup_viability_message: "startup probe failed"
    });

    render(<App />);

    const start = await screen.findByRole("button", { name: "Start Simulation" });
    expect(start).toBeDisabled();
    expect(await screen.findByText(/Startup draft is non-viable/i)).toBeInTheDocument();
  });

  it("marks pending restart after startup draft update while running", async () => {
    setMockStatus({
      phase: "running",
      run_id: 1,
      seed: 42
    });

    render(<App />);

    const slider = await screen.findByLabelText(/Initial population/i);
    fireEvent.change(slider, { target: { value: "320" } });

    await waitFor(() => {
      expect(screen.getByText("Yes")).toBeInTheDocument();
    });
    expect(getMockStatus().pending_restart).toBe(true);
  });

  it("disables runtime controls in idle", async () => {
    render(<App />);
    const tps = await screen.findByLabelText(/Ticks \/ second/i);
    expect(tps).toBeDisabled();
  });

  it("shows API error message when status fetch fails", async () => {
    server.use(
      http.get("http://127.0.0.1:4000/simulation/status", () =>
        HttpResponse.json({ message: "status unavailable" }, { status: 500 })
      )
    );

    render(<App />);

    expect(await screen.findByText("status unavailable")).toBeInTheDocument();
  });

  it("renders runtime mutation controls when simulation is active", async () => {
    setMockStatus({
      phase: "running",
      run_id: 1,
      seed: 42
    });

    render(<App />);

    const weightRate = await screen.findByLabelText(/Weight mutation rate/i);
    expect(weightRate).toBeEnabled();
  });

  it("renders startup world wrap control", async () => {
    render(<App />);
    expect(await screen.findByLabelText(/World wrap/i)).toBeInTheDocument();
  });

  it("renders density-threshold food controls", async () => {
    render(<App />);
    expect(await screen.findByLabelText(/^Food spread threshold:/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/^Food spawn floor density:/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/^Sensor radius:/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/^Runtime food spread threshold:/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/^Runtime food spawn floor:/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/^Runtime sensor radius:/i)).toBeInTheDocument();
  });

  it("renders startup world size controls", async () => {
    render(<App />);
    expect(await screen.findByLabelText(/World width/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/World height/i)).toBeInTheDocument();
  });

  it("renders advanced stage1 config controls", async () => {
    render(<App />);
    expect(await screen.findByLabelText(/Max creatures/i)).toBeInTheDocument();
    expect(await screen.findByLabelText(/Food max density/i)).toBeInTheDocument();
  });

  it("shows creature inspector details after clicking near a creature", async () => {
    const mockStore = {
      frame: {
        tick: 7,
        width: 20,
        height: 20,
        food: new Uint8Array(400),
        barrier_bits: new Uint8Array(50),
        creatures: [
          {
            id: 99,
            lineage_id: 10,
            parent_id: 3,
            x: 1,
            y: 1,
            energy: 0.75,
            age: 12,
            generation: 2,
            node_count: 11
          }
        ],
        population: 1,
        average_energy: 0.75
      },
      status: {
        phase: "running",
        run_id: 1,
        seed: 42,
        tick: 7,
        population: 1,
        average_energy: 0.75,
        pending_restart: false,
        startup_viable: true,
        startup_viability_code: null,
        startup_viability_message: null,
        startup_draft: {
          initial_creatures: 300,
          max_creatures: 5000,
          width: 400,
          height: 400,
          sensor_radius: 12,
          initial_food_density: 0.25,
          energy_initial: 0.7,
          food_spawn_rate: 0.1,
          food_growth_rate: 0.2,
          food_spread_threshold: 0.75,
          food_spawn_floor_density: 0.03,
          energy_per_tick_decay: 0.01,
          energy_per_move: 0.02,
          world_wrap: true
        }
      },
      startupDraft: {
        initial_creatures: 300,
        max_creatures: 5000,
        width: 400,
        height: 400,
        sensor_radius: 12,
        initial_food_density: 0.25,
        energy_initial: 0.7,
        food_spawn_rate: 0.1,
        food_growth_rate: 0.2,
        food_spread_threshold: 0.75,
        food_spawn_floor_density: 0.03,
        energy_per_tick_decay: 0.01,
        energy_per_move: 0.02,
        world_wrap: true
      },
      runtimeConfig: {
        paused: false,
        ticks_per_second: 30,
        sensor_radius: 12,
        food_spawn_rate: 0.1,
        food_growth_rate: 0.2,
        food_spread_threshold: 0.75,
        food_spawn_floor_density: 0.03,
        food_max_density: 1.0,
        food_energy_value: 0.35,
        energy_per_tick_decay: 0.01,
        energy_per_move: 0.02,
        energy_per_compute_node: 0.005,
        energy_per_reproduce: 0.12,
        energy_max: 1.5,
        min_reproduce_energy: 1.0,
        offspring_energy_fraction: 0.45,
        max_creatures: 5000,
        weight_mutation_rate: 0.08,
        weight_mutation_magnitude: 0.18,
        logic_node_mutation_rate: 0.01,
        structural_mutation_rate: 0.02
      },
      serverReachable: true,
      wsConnected: true,
      error: null,
      busyAction: null,
      paintModeEnabled: false,
      paintAllowed: true,
      paintTool: "food",
      brushHalfExtent: 0,
      idlePreviewMode: "paint_layer",
      lastPaintStats: null,
      phase: "running",
      tick: 7,
      population: 1,
      averageEnergy: 0.75,
      startDisabled: false,
      loadStatus: vi.fn(),
      setError: vi.fn(),
      startSimulation: vi.fn(),
      restartSimulation: vi.fn(),
      updateStartupField: vi.fn(),
      togglePause: vi.fn(),
      updateRuntimeField: vi.fn(),
      exportSnapshot: vi.fn().mockResolvedValue("{}"),
      importSnapshot: vi.fn(),
      setPaintModeEnabled: vi.fn(),
      setPaintTool: vi.fn(),
      setBrushHalfExtent: vi.fn(),
      setIdlePreviewMode: vi.fn(),
      commitPaintStroke: vi.fn(),
      clearPaint: vi.fn(),
      refreshPaintPreview: vi.fn(),
      fetchCreatureDetail: vi.fn().mockResolvedValue({
        id: 99,
        lineage_id: 10,
        parent_id: 3,
        x: 1,
        y: 1,
        energy: 0.75,
        age: 12,
        generation: 2,
        node_count: 11,
        last_move_blocked: false,
        last_inputs: {
          food_here: 0.2,
          energy: 0.5,
          random: 0.0,
          food_direction: 0.1,
          food_distance: 0.4,
          creature_direction: -0.2,
          creature_distance: 0.7,
          local_density: 0.35,
          barrier_direction: 0.05,
          barrier_distance: 0.6,
          move_blocked_last_tick: 0,
          memory_read: 0
        },
        last_outputs: {
          move_x: 0.8,
          move_y: -0.1,
          eat: 0.4,
          reproduce: 0.2,
          memory_write: 0
        },
        events: [{ kind: "Moved", tick: 7 }]
      })
    };

    const spy = vi
      .spyOn(simulationStoreModule, "useSimulationStore")
      .mockReturnValue(mockStore as ReturnType<typeof simulationStoreModule.useSimulationStore>);
    try {
      render(<App />);
      const canvas = document.querySelector("canvas.world-canvas") as HTMLCanvasElement;
      Object.defineProperty(canvas, "getBoundingClientRect", {
        value: () => ({
          left: 0,
          top: 0,
          width: 300,
          height: 300,
          right: 300,
          bottom: 300,
          x: 0,
          y: 0,
          toJSON: () => ({})
        })
      });

      fireEvent.click(canvas, { clientX: 7, clientY: 7 });

      expect(await screen.findByText(/Creature Inspector/i)).toBeInTheDocument();
      expect(screen.getByText("99")).toBeInTheDocument();
      expect(screen.getByText("11")).toBeInTheDocument();
      expect(await screen.findByText(/Creature direction/i)).toBeInTheDocument();
      expect(await screen.findByText(/Barrier direction/i)).toBeInTheDocument();
    } finally {
      spy.mockRestore();
    }
  });

  it("renders snapshot import export controls", async () => {
    render(<App />);
    expect(await screen.findByRole("button", { name: /Export Snapshot/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Import Snapshot/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/Snapshot JSON/i)).toBeInTheDocument();
  });

  it("shows paint toolbar controls when paint mode is enabled", async () => {
    render(<App />);
    const toggle = await screen.findByRole("button", { name: "Paint" });
    fireEvent.click(toggle);

    expect(await screen.findByRole("button", { name: "Food" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Erase Barrier" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear Paint" })).toBeInTheDocument();
  });

  it("wires idle preview switch and clear paint actions", async () => {
    const setIdlePreviewMode = vi.fn();
    const refreshPaintPreview = vi.fn();
    const clearPaint = vi.fn();
    const mockStore = {
      frame: {
        tick: 0,
        width: 20,
        height: 20,
        food: new Uint8Array(400),
        barrier_bits: new Uint8Array(50),
        creatures: [],
        population: 0,
        average_energy: 0
      },
      status: {
        phase: "idle",
        run_id: null,
        seed: null,
        tick: 0,
        population: 0,
        average_energy: 0,
        pending_restart: false,
        startup_viable: true,
        startup_viability_code: null,
        startup_viability_message: null,
        startup_draft: {
          initial_creatures: 300,
          max_creatures: 5000,
          width: 400,
          height: 400,
          sensor_radius: 12,
          initial_food_density: 0.25,
          energy_initial: 0.7,
          food_spawn_rate: 0.1,
          food_growth_rate: 0.2,
          food_spread_threshold: 0.75,
          food_spawn_floor_density: 0.03,
          energy_per_tick_decay: 0.01,
          energy_per_move: 0.02,
          world_wrap: true
        }
      },
      startupDraft: {
        initial_creatures: 300,
        max_creatures: 5000,
        width: 400,
        height: 400,
        sensor_radius: 12,
        initial_food_density: 0.25,
        energy_initial: 0.7,
        food_spawn_rate: 0.1,
        food_growth_rate: 0.2,
        food_spread_threshold: 0.75,
        food_spawn_floor_density: 0.03,
        energy_per_tick_decay: 0.01,
        energy_per_move: 0.02,
        world_wrap: true
      },
      runtimeConfig: {
        paused: false,
        ticks_per_second: 30,
        sensor_radius: 12,
        food_spawn_rate: 0.1,
        food_growth_rate: 0.2,
        food_spread_threshold: 0.75,
        food_spawn_floor_density: 0.03,
        food_max_density: 1.0,
        food_energy_value: 0.35,
        energy_per_tick_decay: 0.01,
        energy_per_move: 0.02,
        energy_per_compute_node: 0.005,
        energy_per_reproduce: 0.12,
        energy_max: 1.5,
        min_reproduce_energy: 1.0,
        offspring_energy_fraction: 0.45,
        max_creatures: 5000,
        weight_mutation_rate: 0.08,
        weight_mutation_magnitude: 0.18,
        logic_node_mutation_rate: 0.01,
        structural_mutation_rate: 0.02
      },
      serverReachable: true,
      wsConnected: true,
      error: null,
      busyAction: null,
      paintModeEnabled: true,
      paintAllowed: true,
      paintTool: "food",
      brushHalfExtent: 0,
      idlePreviewMode: "paint_layer",
      lastPaintStats: null,
      phase: "idle",
      tick: 0,
      population: 0,
      averageEnergy: 0,
      startDisabled: false,
      loadStatus: vi.fn(),
      setError: vi.fn(),
      startSimulation: vi.fn(),
      restartSimulation: vi.fn(),
      updateStartupField: vi.fn(),
      togglePause: vi.fn(),
      updateRuntimeField: vi.fn(),
      exportSnapshot: vi.fn().mockResolvedValue("{}"),
      importSnapshot: vi.fn(),
      fetchCreatureDetail: vi.fn().mockResolvedValue(null),
      setPaintModeEnabled: vi.fn(),
      setPaintTool: vi.fn(),
      setBrushHalfExtent: vi.fn(),
      setIdlePreviewMode,
      commitPaintStroke: vi.fn(),
      clearPaint,
      refreshPaintPreview
    };

    const spy = vi
      .spyOn(simulationStoreModule, "useSimulationStore")
      .mockReturnValue(mockStore as ReturnType<typeof simulationStoreModule.useSimulationStore>);
    try {
      render(<App />);
      fireEvent.click(await screen.findByRole("button", { name: "Full Startup" }));
      fireEvent.click(screen.getByRole("button", { name: "Clear Paint" }));

      expect(setIdlePreviewMode).toHaveBeenCalledWith("full_startup");
      expect(clearPaint).toHaveBeenCalled();
    } finally {
      spy.mockRestore();
    }
  });
});
