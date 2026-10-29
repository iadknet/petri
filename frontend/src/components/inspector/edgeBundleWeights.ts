import type { LearnerSource } from "../../types/generated/LearnerSource.ts";
import type { MemberPolicy } from "../../types/generated/MemberPolicy.ts";

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

/** A member's policy and `credits` as the inspector badges print them (T25.F12). */
export function policyLabel(policy: MemberPolicy, credits: boolean): string {
	return credits ? `${policy} · credits` : policy;
}

/** Where a learner's learning is stored, as the badges print it (T25.F12). */
export function sourceLabel(source: LearnerSource): string {
	return source === "member_default" ? "member default" : source;
}
