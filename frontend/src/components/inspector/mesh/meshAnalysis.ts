import type { CreatureGenome, NodeGenome, RouteTarget } from "../../../types/genome.ts";

export type MeshBackendKind = "vm" | "graph";
export type MeshBackendFilter = "all" | MeshBackendKind;
export type MeshClosureDirection = "upstream" | "downstream";

export interface MeshAnalyzedNode {
	id: number;
	node: NodeGenome;
	backendKind: MeshBackendKind;
	isEntry: boolean;
	reachable: boolean;
	incomingIds: number[];
	outgoingIds: number[];
}

export interface MeshAnalyzedEdge {
	id: string;
	fromId: number;
	toId: number;
	isDangling: boolean;
}

export interface MeshAnalysis {
	entryNodeId: number;
	topologyKey: string;
	nodes: MeshAnalyzedNode[];
	nodesById: Map<number, MeshAnalyzedNode>;
	edges: MeshAnalyzedEdge[];
	reachableNodeIds: Set<number>;
	unreachableNodeIds: Set<number>;
	danglingTargetIds: Set<number>;
}

export function getMeshEdgeOccurrenceIndex(targets: RouteTarget[], targetIndex: number): number {
	const targetId = targets[targetIndex]?.target_id;
	if (targetId === undefined) {
		return 0;
	}

	let occurrenceIndex = 0;
	for (let index = 0; index < targetIndex; index += 1) {
		if (targets[index]?.target_id === targetId) {
			occurrenceIndex += 1;
		}
	}
	return occurrenceIndex;
}

export function formatMeshEdgeId(fromId: number, toId: number, occurrenceIndex = 0): string {
	return occurrenceIndex === 0 ? `${fromId}->${toId}` : `${fromId}->${toId}#${occurrenceIndex}`;
}

export function analyzeMesh(genome: CreatureGenome): MeshAnalysis {
	const sortedNodes = [...genome.nodes].toSorted((left, right) => left.node_id - right.node_id);
	const nodesById = new Map<number, MeshAnalyzedNode>();
	const reachableNodeIds = collectReachableNodeIds(genome);
	const incomingMap = new Map<number, number[]>();
	const danglingTargetIds = new Set<number>();
	const edges: MeshAnalyzedEdge[] = [];

	for (const node of sortedNodes) {
		nodesById.set(node.node_id, {
			id: node.node_id,
			node,
			backendKind: getBackendKind(node),
			isEntry: node.node_id === genome.entry_node_id,
			reachable: reachableNodeIds.has(node.node_id),
			incomingIds: [],
			outgoingIds: node.targets.map((t) => t.target_id),
		});
	}

	for (const node of sortedNodes) {
		for (const [targetIndex, routeTarget] of node.targets.entries()) {
			const targetId = routeTarget.target_id;
			const target = nodesById.get(targetId);
			if (!target) {
				danglingTargetIds.add(targetId);
			}
			const incoming = incomingMap.get(targetId) ?? [];
			incoming.push(node.node_id);
			incomingMap.set(targetId, incoming);
			edges.push({
				id: formatMeshEdgeId(
					node.node_id,
					targetId,
					getMeshEdgeOccurrenceIndex(node.targets, targetIndex),
				),
				fromId: node.node_id,
				toId: targetId,
				isDangling: !target,
			});
		}
	}

	const nodes = sortedNodes.map((node) => {
		const analyzedNode = nodesById.get(node.node_id);
		if (!analyzedNode) {
			throw new Error(`missing analyzed node for ${node.node_id}`);
		}
		const incomingIds =
			incomingMap.get(node.node_id)?.toSorted((left, right) => left - right) ?? [];
		analyzedNode.incomingIds = incomingIds;
		analyzedNode.outgoingIds = node.targets
			.map((t) => t.target_id)
			.toSorted((left, right) => left - right);
		return analyzedNode;
	});

	return {
		entryNodeId: genome.entry_node_id,
		topologyKey: buildTopologyKey(sortedNodes),
		nodes,
		nodesById,
		edges,
		reachableNodeIds,
		unreachableNodeIds: new Set(
			nodes.filter((node) => !reachableNodeIds.has(node.id)).map((node) => node.id),
		),
		danglingTargetIds,
	};
}

export function collectMeshClosure(
	analysis: MeshAnalysis,
	startNodeId: number,
	direction: MeshClosureDirection,
): Set<number> {
	if (!analysis.nodesById.has(startNodeId)) {
		return new Set();
	}

	const visited = new Set<number>();
	const queue = [startNodeId];

	while (queue.length > 0) {
		const currentId = queue.shift();
		if (currentId === undefined || visited.has(currentId)) {
			continue;
		}
		visited.add(currentId);

		const node = analysis.nodesById.get(currentId);
		if (!node) {
			continue;
		}

		const nextIds = direction === "upstream" ? node.incomingIds : node.outgoingIds;
		for (const nextId of nextIds) {
			if (analysis.nodesById.has(nextId) && !visited.has(nextId)) {
				queue.push(nextId);
			}
		}
	}

	return visited;
}

export function matchesBackendFilter(
	node: Pick<MeshAnalyzedNode, "backendKind">,
	filter: MeshBackendFilter,
): boolean {
	return filter === "all" ? true : node.backendKind === filter;
}

function collectReachableNodeIds(genome: CreatureGenome): Set<number> {
	const nodeMap = new Map(genome.nodes.map((node) => [node.node_id, node] as const));
	const visited = new Set<number>();
	const queue = [genome.entry_node_id];

	while (queue.length > 0) {
		const nodeId = queue.shift();
		if (nodeId === undefined || visited.has(nodeId)) {
			continue;
		}
		visited.add(nodeId);
		const node = nodeMap.get(nodeId);
		if (!node) {
			continue;
		}
		for (const routeTarget of node.targets) {
			if (nodeMap.has(routeTarget.target_id) && !visited.has(routeTarget.target_id)) {
				queue.push(routeTarget.target_id);
			}
		}
	}

	return visited;
}

function buildTopologyKey(nodes: NodeGenome[]): string {
	return nodes
		.map(
			(node) =>
				`${node.node_id}:${node.targets
					.map((t) => t.target_id)
					.toSorted((left, right) => left - right)
					.join(",")}`,
		)
		.join("|");
}

function getBackendKind(node: NodeGenome): MeshBackendKind {
	return "Vm" in node.backend_def ? "vm" : "graph";
}
