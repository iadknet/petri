import { afterEach, describe, expect, it, vi } from "vitest";
import { annotation } from "../../../test/brainFixtures.ts";
import type { BrainView } from "../../../types/generated/BrainView.ts";
import { analyzeMesh } from "./meshAnalysis.ts";
import { createElk, layoutMesh } from "./meshLayout.ts";

function makeCyclicGenome(): BrainView {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				annotation: annotation(),
				input_refs: [],
				targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
				backend: {
					Vm: {
						register_count: 1,
						constants: [],
						program: ["Halt"],
					},
				},
			},
			{
				node_id: 2,
				annotation: annotation(),
				input_refs: [],
				targets: [{ target_id: 1, slot: 0, gate_bias: 0.0 }],
				backend: {
					Graph: {
						groups: [],
						compute_nodes: [{ kind: { Constant: 1 }, edges: [] }],
						output_sinks: [],
					},
				},
			},
			{
				node_id: 9,
				annotation: annotation({ reachable: false }),
				input_refs: [],
				targets: [{ target_id: 42, slot: 0, gate_bias: 0.0 }],
				backend: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
					},
				},
			},
		],
	};
}

describe("layoutMesh", () => {
	it("lays out cyclic reachable graphs and trailing unreachable clusters", async () => {
		const layout = await layoutMesh(analyzeMesh(makeCyclicGenome()));
		const node1 = layout.nodesById.get(1);
		const node2 = layout.nodesById.get(2);
		const node9 = layout.nodesById.get(9);

		expect(node1).toBeDefined();
		expect(node2).toBeDefined();
		expect(node9).toBeDefined();
		expect(node9?.x ?? 0).toBeGreaterThan(Math.max(node1?.x ?? 0, node2?.x ?? 0));
		expect(layout.width).toBeGreaterThan(0);
		expect(layout.height).toBeGreaterThan(0);
	});

	it("drops routed geometry for missing targets without failing layout", async () => {
		const layout = await layoutMesh(analyzeMesh(makeCyclicGenome()));

		expect(layout.edges.some((edge) => edge.toId === 42)).toBe(false);
		expect(layout.edges.some((edge) => edge.fromId === 1 && edge.toId === 2)).toBe(true);
	});
});

describe("createElk", () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("runs ELK in a Web Worker when the browser provides one", async () => {
		const workerUrls: string[] = [];
		class FakeWorker extends EventTarget {
			onmessage: ((event: MessageEvent) => void) | null = null;
			constructor(url: string | URL) {
				super();
				workerUrls.push(String(url));
			}
			postMessage() {}
			terminate() {}
		}
		vi.stubGlobal("Worker", FakeWorker);

		await createElk();

		expect(workerUrls).toHaveLength(1);
		expect(workerUrls[0]).toContain("elk-worker");
	});

	it("falls back to main-thread ELK when Worker is unavailable", async () => {
		vi.stubGlobal("Worker", undefined);

		const elk = await createElk();
		const result = await elk.layout({
			id: "root",
			children: [{ id: "a", width: 10, height: 10 }],
			edges: [],
		});

		expect(result.children?.[0]?.x).toBeDefined();
	});
});

describe("ELK worker failure", () => {
	/** A worker that never answers and reports a load failure on demand. */
	class FailingWorker extends EventTarget {
		static instances: FailingWorker[] = [];
		onmessage: ((event: MessageEvent) => void) | null = null;
		terminated = false;
		constructor(_url: string | URL) {
			super();
			FailingWorker.instances.push(this);
		}
		postMessage() {}
		terminate() {
			this.terminated = true;
		}
		fail() {
			this.dispatchEvent(new Event("error"));
		}
	}

	async function freshModule() {
		vi.resetModules();
		return import("./meshLayout.ts");
	}

	afterEach(() => {
		FailingWorker.instances = [];
		ReplyingWorker.instances = [];
		vi.unstubAllGlobals();
	});

	it("rejects pending layout work and terminates the worker when it fails to load", async () => {
		// Arrange
		vi.stubGlobal("Worker", FailingWorker);
		const { createElk: freshCreateElk } = await freshModule();
		const elk = await freshCreateElk();
		const pending = elk.layout({ id: "root", children: [], edges: [] });

		// Act
		FailingWorker.instances[0]!.fail();

		// Assert
		await expect(pending).rejects.toThrow(/worker/i);
		expect(FailingWorker.instances[0]!.terminated).toBe(true);
	});

	it("recovers on the next layout after the worker fails", async () => {
		// Arrange
		vi.stubGlobal("Worker", FailingWorker);
		const { layoutMesh: freshLayoutMesh } = await freshModule();
		const analysis = analyzeMesh(makeCyclicGenome());
		const first = freshLayoutMesh(analysis);
		await vi.waitFor(() => expect(FailingWorker.instances).toHaveLength(1));

		// Act
		FailingWorker.instances[0]!.fail();
		await expect(first).rejects.toThrow(/worker/i);
		const layout = await freshLayoutMesh(analysis);

		// Assert: the cached failed instance was dropped and layout completes.
		expect(layout.nodesById.get(1)).toBeDefined();
		expect(layout.width).toBeGreaterThan(0);
	});

	/** A worker that answers each layout request, holding replies while `hold` is set. */
	class ReplyingWorker extends EventTarget {
		static instances: ReplyingWorker[] = [];
		onmessage: ((event: MessageEvent) => void) | null = null;
		hold = false;
		held: number[] = [];
		constructor(_url: string | URL) {
			super();
			ReplyingWorker.instances.push(this);
		}
		postMessage(message: { id: number; cmd: string }) {
			if (message.cmd === "layout" && this.hold) {
				this.held.push(message.id);
				return;
			}
			this.reply(message.id);
		}
		reply(id: number) {
			queueMicrotask(() =>
				this.onmessage?.(
					new MessageEvent("message", { data: { id, data: { id: "root", width: 1, height: 1 } } }),
				),
			);
		}
		terminate() {}
		fail() {
			this.dispatchEvent(new Event("error"));
		}
	}

	it("releases failure handlers once layouts settle and only rejects in-flight work", async () => {
		// Arrange
		vi.stubGlobal("Worker", ReplyingWorker);
		const { createElk: freshCreateElk, pendingWorkerLayoutCount } = await freshModule();
		const elk = await freshCreateElk();
		const graph = { id: "root", children: [], edges: [] };
		const settled = await Promise.all([elk.layout(graph), elk.layout(graph), elk.layout(graph)]);
		const worker = ReplyingWorker.instances[0]!;
		worker.hold = true;
		const inFlight = elk.layout(graph);
		const countWhileInFlight = pendingWorkerLayoutCount(elk);
		worker.hold = false;

		// Act
		worker.fail();

		// Assert: settled layouts left no handlers behind; only the in-flight one is rejected.
		expect(settled).toHaveLength(3);
		expect(countWhileInFlight).toBe(1);
		await expect(inFlight).rejects.toThrow(/worker/i);
		expect(pendingWorkerLayoutCount(elk)).toBe(0);
	});
});
