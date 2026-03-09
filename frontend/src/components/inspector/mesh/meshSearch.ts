import type { MeshSemantics } from "./meshSemantics.ts";
import { tokenizeSearchText } from "./meshSemantics.ts";

export interface MeshSearchResult {
	nodeId: number;
	score: number;
}

export function searchMeshSemantics(semantics: MeshSemantics, query: string): MeshSearchResult[] {
	const normalizedQuery = query.trim().toLowerCase();
	if (!normalizedQuery) {
		return semantics.nodes.map((node) => ({ nodeId: node.nodeId, score: 0 }));
	}

	const queryTokens = tokenizeSearchText(normalizedQuery);
	const results: MeshSearchResult[] = [];

	for (const node of semantics.nodes) {
		let score = 0;
		let matchedAllTokens = queryTokens.length > 0;

		if (String(node.nodeId) === normalizedQuery || `#${node.nodeId}` === normalizedQuery) {
			score += 200;
		}
		if (
			node.label.toLowerCase() === normalizedQuery ||
			node.shortLabel.toLowerCase() === normalizedQuery
		) {
			score += 180;
		}

		for (const token of queryTokens) {
			if (node.searchTokens.includes(token)) {
				score += 80;
				continue;
			}
			if (node.searchText.includes(token)) {
				score += 30;
				continue;
			}
			matchedAllTokens = false;
			break;
		}

		if (!matchedAllTokens && score === 0) {
			continue;
		}

		results.push({
			nodeId: node.nodeId,
			score: score + (node.reachable ? 5 : 0),
		});
	}

	return results.toSorted((left, right) => {
		if (right.score !== left.score) {
			return right.score - left.score;
		}
		return left.nodeId - right.nodeId;
	});
}
