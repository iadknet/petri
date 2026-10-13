import type { ElkExtendedEdge, ELK as ElkInstance, ElkNode, ElkPoint } from "elkjs/lib/elk-api";
import ELK from "elkjs/lib/elk-api.js";
import elkWorkerUrl from "elkjs/lib/elk-worker.min.js?url";
import type { MeshAnalysis } from "./meshAnalysis.ts";

let elkInstance: Promise<ElkInstance> | null = null;
/** Set once the layout worker has failed; later instances run on the main thread. */
let workerFailed = false;

/**
 * Runs ELK in a Web Worker so layout never blocks the main thread. Without
 * Worker support (jsdom, old runtimes), or after the worker has failed, it
 * falls back to the bundled main-thread build, loaded on demand so it stays
 * out of the main chunk.
 */
export async function createElk(onWorkerFailure?: () => void): Promise<ElkInstance> {
	if (typeof Worker !== "undefined" && !workerFailed) {
		return createWorkerElk(onWorkerFailure);
	}
	const { default: BundledElk } = await import("elkjs/lib/elk.bundled.js");
	return new BundledElk();
}

/**
 * ELK's worker wrapper only listens for messages, so a worker that fails to
 * load would leave every layout pending forever. Reject in-flight and later
 * layouts on worker error and terminate the worker.
 */
function createWorkerElk(onFailure?: () => void): ElkInstance {
	const worker = new Worker(elkWorkerUrl);
	const elk = new ELK({ workerUrl: elkWorkerUrl, workerFactory: () => worker });
	let rejectFailure: (error: Error) => void = () => {};
	const failure = new Promise<never>((_resolve, reject) => {
		rejectFailure = reject;
	});
	failure.catch(() => {});
	let failed = false;
	const fail = () => {
		if (failed) return;
		failed = true;
		workerFailed = true;
		elk.terminateWorker();
		rejectFailure(new Error("The ELK layout worker failed"));
		onFailure?.();
	};
	worker.addEventListener("error", fail);
	worker.addEventListener("messageerror", fail);
	const layout = elk.layout.bind(elk);
	elk.layout = ((graph, args) =>
		Promise.race([layout(graph, args), failure])) as ElkInstance["layout"];
	return elk;
}

function getElk(): Promise<ElkInstance> {
	if (!elkInstance) {
		const created: Promise<ElkInstance> = createElk(() => {
			if (elkInstance === created) elkInstance = null;
		});
		elkInstance = created;
	}
	return elkInstance;
}

export const MESH_NODE_WIDTH = 192;
export const MESH_NODE_HEIGHT = 64;
const REGION_GAP = 88;

export interface MeshLayoutNode {
	id: number;
	x: number;
	y: number;
	width: number;
	height: number;
	region: "reachable" | "unreachable";
}

export interface MeshLayoutEdge {
	id: string;
	fromId: number;
	toId: number;
	points: ElkPoint[];
}

export interface MeshLayout {
	topologyKey: string;
	nodes: MeshLayoutNode[];
	nodesById: Map<number, MeshLayoutNode>;
	edges: MeshLayoutEdge[];
	width: number;
	height: number;
}

interface RegionLayout {
	nodes: MeshLayoutNode[];
	edges: MeshLayoutEdge[];
	width: number;
	height: number;
}

export async function layoutMesh(analysis: MeshAnalysis): Promise<MeshLayout> {
	const reachableLayout = await layoutRegion(
		analysis,
		analysis.nodes.filter((node) => node.reachable).map((node) => node.id),
		"reachable",
		0,
	);
	const unreachableOffset = reachableLayout.width > 0 ? reachableLayout.width + REGION_GAP : 0;
	const unreachableLayout = await layoutRegion(
		analysis,
		analysis.nodes.filter((node) => !node.reachable).map((node) => node.id),
		"unreachable",
		unreachableOffset,
	);

	const nodes = [...reachableLayout.nodes, ...unreachableLayout.nodes];
	const nodesById = new Map(nodes.map((node) => [node.id, node] as const));
	const edges = [...reachableLayout.edges, ...unreachableLayout.edges];
	const width = Math.max(
		reachableLayout.width,
		unreachableLayout.width + unreachableOffset,
		nodes.reduce((maxX, node) => Math.max(maxX, node.x + node.width), 0),
	);
	const height = Math.max(reachableLayout.height, unreachableLayout.height, 0);

	return {
		topologyKey: analysis.topologyKey,
		nodes,
		nodesById,
		edges,
		width,
		height,
	};
}

async function layoutRegion(
	analysis: MeshAnalysis,
	nodeIds: number[],
	region: "reachable" | "unreachable",
	xOffset: number,
): Promise<RegionLayout> {
	if (nodeIds.length === 0) {
		return {
			nodes: [],
			edges: [],
			width: 0,
			height: 0,
		};
	}

	const nodeIdSet = new Set(nodeIds);
	const elkGraph: ElkNode = {
		id: `${region}-root`,
		layoutOptions: {
			"elk.algorithm": "layered",
			"elk.direction": "RIGHT",
			"elk.edgeRouting": "ORTHOGONAL",
			"elk.padding": "[left=24,top=24,right=24,bottom=24]",
			"elk.spacing.nodeNode": "36",
			"elk.layered.spacing.nodeNodeBetweenLayers": "56",
			"elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
		},
		children: nodeIds
			.toSorted((left, right) => left - right)
			.map((nodeId) => ({
				id: `${nodeId}`,
				width: MESH_NODE_WIDTH,
				height: MESH_NODE_HEIGHT,
			})),
		edges: analysis.edges
			.filter((edge) => !edge.isDangling && nodeIdSet.has(edge.fromId) && nodeIdSet.has(edge.toId))
			.map(
				(edge): ElkExtendedEdge => ({
					id: edge.id,
					sources: [`${edge.fromId}`],
					targets: [`${edge.toId}`],
				}),
			),
	};

	const layoutResult = await (await getElk()).layout(elkGraph);
	const nodes = (layoutResult.children ?? []).map((child) => ({
		id: Number(child.id),
		x: (child.x ?? 0) + xOffset,
		y: child.y ?? 0,
		width: child.width ?? MESH_NODE_WIDTH,
		height: child.height ?? MESH_NODE_HEIGHT,
		region,
	}));
	const edges = (layoutResult.edges ?? []).flatMap((edge) =>
		edge.sections?.length
			? [
					{
						id: edge.id ?? "edge",
						fromId: Number(edge.sources[0]),
						toId: Number(edge.targets[0]),
						points: flattenEdgePoints(edge.sections, xOffset),
					},
				]
			: [],
	);

	return {
		nodes,
		edges,
		width: layoutResult.width ?? 0,
		height: layoutResult.height ?? 0,
	};
}

function flattenEdgePoints(
	sections: NonNullable<ElkExtendedEdge["sections"]>,
	xOffset: number,
): ElkPoint[] {
	const points: ElkPoint[] = [];

	for (const section of sections) {
		points.push(offsetPoint(section.startPoint, xOffset));
		for (const bendPoint of section.bendPoints ?? []) {
			points.push(offsetPoint(bendPoint, xOffset));
		}
		points.push(offsetPoint(section.endPoint, xOffset));
	}

	return points;
}

function offsetPoint(point: ElkPoint, xOffset: number): ElkPoint {
	return {
		x: point.x + xOffset,
		y: point.y,
	};
}
