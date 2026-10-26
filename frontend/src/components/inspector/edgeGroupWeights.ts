import type { LearnerScope } from "../../types/generated/LearnerScope.ts";
import type { Scope } from "../../types/generated/Scope.ts";

/** The composition the brain view resolved for a member, as printed:
 * `scale × pattern + correction = genomic`, every term the server's. */
export function composeTerms(terms: {
	scale: number;
	pattern: number;
	own: number;
	genomic: number;
}): string {
	return `${terms.scale.toFixed(3)} × ${terms.pattern.toFixed(2)} + ${terms.own.toFixed(3)} = ${terms.genomic.toFixed(3)}`;
}

/** A member's or a learner's scope as the inspector badges print it (T25.F07). */
export function scopeLabel(scope: Scope | LearnerScope): string {
	return `${scope.toLowerCase()} scope`;
}
