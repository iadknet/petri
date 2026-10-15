// petri-viz: Petri node-inspector-style explainers for inline show_widget visuals.
// Source of truth. `sh build.sh` writes petri-viz.min.js, the copy pasted into widgets.
// Colors, labels and compute math mirror the frontend inspector and v3-core;
// frontend/src/components/inspector/petriVizDrift.test.ts fails when they drift.
(function (root) {
	"use strict";
	const DN = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
	// GraphInternalsNode.tsx CATEGORY_COLORS as [r, g, b, bgAlpha, borderAlpha, text].
	const CAT = {
		input: [16, 185, 129, 0.15, 0.4, "#6ee7b7"],
		arithmetic: [148, 163, 184, 0.12, 0.3, "#cbd5e1"],
		activation: [56, 189, 248, 0.12, 0.35, "#7dd3fc"],
		logic: [167, 139, 250, 0.12, 0.35, "#c4b5fd"],
		stateful: [196, 130, 250, 0.12, 0.35, "#d8b4fe"],
		constant: [100, 116, 139, 0.12, 0.35, "#94a3b8"],
		output_value: [251, 191, 36, 0.15, 0.4, "#fcd34d"],
		output_action: [249, 115, 22, 0.15, 0.4, "#fdba74"],
		// Not in the inspector: a proposed mechanism that does not exist yet.
		proposal: [45, 212, 191, 0.14, 0.55, "#5eead4"],
	};
	// graphFlowAdapter.ts EDGE_TINTS; compute-to-compute (cc) falls back to slate;
	// gm is an edge-group member (group_member).
	const TINT = { ic: [16, 185, 129], io: [251, 191, 36], co: [251, 191, 36], cc: [148, 163, 184], gm: [192, 132, 252] };
	// graphNodeFormatters.ts labels and graphNodeCategories.ts sets.
	const STR = { WeightedSum: "Σ Sum", Clamp01: "Clamp", GreaterThan: "A > B" };
	const PARAM = { Constant: "Const", Threshold: "Thresh", DecayIntegrator: "Decay", Momentum: "Momentum", Oscillator: "Osc" };
	const KCAT = { Constant: "constant", DecayIntegrator: "stateful", Momentum: "stateful", Oscillator: "stateful", AdaptiveGain: "stateful", Sigmoid: "activation", Tanh: "activation", Relu: "activation", Clamp01: "activation", Threshold: "activation", GreaterThan: "logic", Select: "logic" };
	const CW = 6.6, MUTED = "#94a3b8", VALUE = "#67e8f9";
	const FONT = `font-family="ui-monospace,Menlo,monospace" font-size="11" dominant-baseline="central"`;

	const obj = () => Object.create(null);
	const own = (o, k) => (o && Object.prototype.hasOwnProperty.call(o, k) ? o[k] : undefined);
	const kindName = (k) => (typeof k === "string" ? k : Object.keys(k)[0]);
	const kindParam = (k) => (typeof k === "string" ? NaN : Number(k[kindName(k)]));
	function kindLabel(k) {
		const n = kindName(k), v = kindParam(k);
		if (typeof k === "string") return own(STR, k) || k;
		if (own(PARAM, n)) return `${PARAM[n]}(${v.toFixed(2)})`;
		return Number.isNaN(v) ? n : `${n}(${Number.isInteger(v) ? v : v.toFixed(2)})`;
	}
	const kindCat = (k) => own(KCAT, kindName(k)) || "arithmetic";
	const clamp = (x, lo, hi) => Math.min(Math.max(x, lo), hi);
	// v3-core runtime/cgp/eval.rs evaluate_compute_kind; w = weighted inputs.
	function evalKind(k, w, st) {
		const p = kindParam(k), s = w.reduce((a, b) => a + b, 0), a = clamp(p, 0, 1);
		const out = {
			Multiply: () => w.reduce((x, y) => x * y, 1),
			Negate: () => -s,
			Abs: () => Math.abs(s),
			Min: () => (w.length ? Math.min(...w) : 0),
			Max: () => (w.length ? Math.max(...w) : 0),
			Threshold: () => +(s > p),
			GreaterThan: () => +((w[0] || 0) > (w[1] || 0)),
			Sigmoid: () => 1 / (1 + Math.exp(-s)),
			Tanh: () => Math.tanh(s),
			Relu: () => Math.max(s, 0),
			Clamp01: () => clamp(s, 0, 1),
			Select: () => ((w[0] || 0) >= 0.5 ? w[1] || 0 : w[2] || 0),
			Constant: () => p,
			DecayIntegrator: () => (st = (1 - a) * st + a * s),
			Momentum: () => (st = a * st + (1 - a) * s),
			Oscillator: () => Math.sin(2 * Math.PI * (st = (st + clamp(p, 0, 8)) % 1)),
			AdaptiveGain: () => (st = clamp(st + 0.01 * s, 0.1, 2)) * s,
		};
		const f = own(out, kindName(k));
		const v = f ? f() : s;
		return { v, state: st };
	}

	const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
	const f2 = (x) => (Math.abs(x) < 0.005 ? 0 : x).toFixed(2);
	const signed = (x) => (x > 0.004 ? "+" : "") + f2(x);
	const rgba = (c, a) => `rgba(${c[0]},${c[1]},${c[2]},${a})`;
	const mix = (c, a) => `rgb(${[2, 6, 23].map((b, i) => Math.round(b + (c[i] - b) * a))})`;
	const txt = (x, y, s, fill, end) => `<text x="${x}" y="${y}" fill="${fill}"${end ? ' text-anchor="end"' : ""} ${FONT}>${s}</text>`;
	const rect = (x, y, w, h, fill, stroke, extra) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="4" fill="${fill}" stroke="${stroke}"${extra || ""}/>`;
	const sum = (a) => a.reduce((x, y) => x + y, 0);

	// Normalize spec nodes: t = in | cn | out, with label, cat and inspector index.
	function prepare(view) {
		let ci = 0, oi = 0;
		const byId = obj();
		const nodes = (view.nodes || []).map((n) => {
			const m = { id: n.id, from: n.from || [], dim: !!n.dim, kind: n.kind };
			if (n.in !== undefined) Object.assign(m, { t: "in", label: n.in, cat: "input" });
			else if (n.out !== undefined) Object.assign(m, { t: "out", label: n.out, cat: n.cat || "output_action", sub: n.sub === undefined ? "action vote" : n.sub, idx: oi++ });
			else Object.assign(m, { t: "cn", label: n.label || (n.kind ? kindLabel(n.kind) : "?"), cat: n.cat || (n.kind ? kindCat(n.kind) : "arithmetic"), sub: n.sub, idx: n.idx === false ? undefined : ci++ });
			return (byId[m.id] = m);
		});
		return { nodes, byId, groups: view.groups || [] };
	}
	// Inspector layering: inputs and constants first, sinks last, compute by depth.
	function depthOf(G) {
		const d = obj(), busy = obj();
		const walk = (n) => {
			if (d[n.id] !== undefined || busy[n.id]) return d[n.id] || 0;
			busy[n.id] = true;
			let best = 0;
			if (n.t === "cn" && !(n.kind && kindName(n.kind) === "Constant"))
				n.from.forEach((f) => G.byId[f[0]] && (best = Math.max(best, walk(G.byId[f[0]]) + 1)));
			busy[n.id] = false;
			return (d[n.id] = best);
		};
		G.nodes.forEach((n) => n.t !== "out" && walk(n));
		const last = Math.max(0, ...G.nodes.filter((n) => n.t !== "out").map((n) => d[n.id])) + 1;
		G.nodes.forEach((n) => n.t === "out" && (d[n.id] = last));
		return d;
	}
	// One example tick, in runtime order (cgp/execute.rs): compute nodes run in
	// listed order, then sinks. A source not yet run this tick (listed later, or
	// the node itself) reads its previous output from tick.prevOut, default 0.
	// tick.values pins any node; tick.prev is operator state, default 0.
	function evaluate(G, tick) {
		const val = Object.assign(obj(), tick.values), done = obj(), change = obj();
		for (const k in val) done[k] = true;
		const read = (id) => (done[id] ? val[id] : own(tick.prevOut, id) || 0);
		const run = (n) => {
			if (done[n.id]) return;
			const w = n.from.map((f) => f[1] * read(f[0]));
			if (n.t === "out" || !n.kind) val[n.id] = sum(w);
			else {
				const prev = own(tick.prev, n.id) || 0, r = evalKind(n.kind, w, prev);
				val[n.id] = r.v;
				if (kindCat(n.kind) === "stateful") change[n.id] = `${f2(prev)}→${f2(r.state)}`;
			}
			done[n.id] = true;
		};
		G.nodes.filter((n) => n.t === "cn").forEach(run);
		G.nodes.filter((n) => n.t === "out").forEach(run);
		return { val, change };
	}
	function layout(G, d, ticks, sort) {
		const cols = [], P = obj();
		let x = 12, maxY = 0;
		G.nodes.forEach((n) => (cols[d[n.id]] = cols[d[n.id]] || []).push(n));
		const width = (n) => {
			let w = (n.idx !== undefined ? String(n.idx).length * CW + 6 : 0) + n.label.length * CW + 16;
			if (n.sub) w = Math.max(w, (n.sub.length + (ticks ? 9 : 0)) * CW + 16);
			else if (ticks) w = Math.max(w, 10 * CW + 16);
			return Math.max(72, Math.ceil(w));
		};
		cols.forEach((col, ci) => {
			if (!col) return;
			const cw = Math.max(...col.map(width));
			if (ci) {
				col.forEach((n) => {
					const ys = n.from.filter((f) => P[f[0]]).map((f) => P[f[0]].y + P[f[0]].h / 2);
					n.b = ys.length ? sum(ys) / ys.length : 0;
				});
				if (sort) col.sort((a, b) => a.b - b.b);
			}
			let y = 0;
			col.forEach((n) => {
				const h = n.sub || ticks ? 38 : 28;
				P[n.id] = { x, y, w: cw, h };
				y += h + 10;
			});
			const shift = ci ? Math.max(12, sum(col.map((n) => n.b)) / col.length - (y - 10) / 2) : G.groups.length ? 32 : 12;
			col.forEach((n) => (P[n.id].y += shift, maxY = Math.max(maxY, P[n.id].y + P[n.id].h)));
			x += cw + 34;
		});
		return { P, W: Math.ceil(x - 22), H: Math.ceil(maxY + 16) };
	}
	function graphSvg(G, L, R, uid, labels) {
		const P = L.P, outDeg = obj(), inDeg = obj();
		let groups = "", edges = "", nodes = "", defs = "";
		G.nodes.forEach((n) => n.from.forEach((f) => (outDeg[f[0]] = (outDeg[f[0]] || 0) + 1, inDeg[n.id] = (inDeg[n.id] || 0) + 1)));
		G.groups.forEach((g) => {
			const bs = g.ids.map((id) => P[id]).filter(Boolean);
			if (!bs.length) return;
			const x0 = Math.min(...bs.map((b) => b.x)) - 6, y0 = Math.min(...bs.map((b) => b.y)) - 22;
			const x1 = Math.max(...bs.map((b) => b.x + b.w)) + 6, y1 = Math.max(...bs.map((b) => b.y + b.h)) + 6;
			groups += `<rect x="${x0}" y="${y0}" width="${x1 - x0}" height="${y1 - y0}" rx="8" fill="none" stroke="${rgba(CAT.proposal, 0.5)}" stroke-dasharray="4 3"/>${txt(x0 + 6, y0 + 10, esc(g.label), CAT.proposal[5])}`;
		});
		G.nodes.forEach((n) => n.from.forEach(([sid, w]) => {
			const s = G.byId[sid], a = P[sid], b = P[n.id];
			if (!s || !a) return;
			const t = (s.t === "in" ? "i" : "c") + (n.t === "out" ? "o" : "c");
			let op = w === 0 ? 0.3 : clamp(Math.abs(w), 0.2, 1);
			if (s.dim || n.dim) op *= 0.35;
			const sx = a.x + a.w, sy = a.y + a.h / 2, tx = b.x - 1, ty = b.y + b.h / 2, k = Math.max((tx - sx) / 2, 24);
			edges += `<path d="M${sx} ${sy} C${sx + k} ${sy},${tx - k} ${ty},${tx} ${ty}" fill="none" stroke="${rgba(TINT[t], op)}"${w === 0 ? ' stroke-dasharray="4 3"' : ""} marker-end="url(#${uid}${t})"/>`;
			if (labels && Math.abs(w - 1) > 0.01) {
				const nearTarget = (outDeg[sid] || 0) > 1 && (inDeg[n.id] || 0) <= outDeg[sid];
				edges += nearTarget ? txt(tx - 10, ty - 8, w.toFixed(2), rgba(TINT[t], 0.8), 1) : txt(sx + 6, sy - 8, w.toFixed(2), rgba(TINT[t], 0.8));
			}
		}));
		G.nodes.forEach((n) => {
			const p = P[n.id], c = own(CAT, n.cat) || CAT.arithmetic, v = R ? R.val[n.id] : undefined;
			let bg = c[3] * (n.dim ? 0.4 : 1), bd = c[4] * (n.dim ? 0.5 : 1), x = p.x + 7;
			if (v !== undefined) (bg += Math.min(Math.abs(v), 1) * 0.25, bd += Math.min(Math.abs(v), 1) * 0.3);
			nodes += rect(p.x, p.y, p.w, p.h, mix(c, bg), rgba(c, bd), n.cat === "constant" || n.dim ? ' stroke-dasharray="3 2"' : "");
			if (n.cat === "stateful") nodes += rect(p.x + 2, p.y + 2, p.w - 4, p.h - 4, "none", rgba(c, 0.3), ' stroke-width="2"');
			if (n.idx !== undefined) (nodes += txt(x, p.y + 14, n.idx, MUTED), x += String(n.idx).length * CW + 6);
			nodes += txt(x, p.y + 14, esc(n.label), n.dim ? rgba(c, 0.55) : c[5]);
			if (n.sub) nodes += txt(p.x + 7, p.y + 27, esc(n.sub), MUTED);
			if (v === undefined) return;
			const shown = R.change[n.id] || (n.t === "out" ? `Σ=${f2(v)}` : `= ${f2(v)}`), color = R.change[n.id] ? "#fbbf24" : VALUE;
			nodes += n.sub ? txt(p.x + p.w - 6, p.y + 27, shown, color, 1) : txt(p.x + 7, p.y + 27, shown, color);
		});
		for (const t in TINT) defs += `<marker id="${uid}${t}" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10z" fill="${rgba(TINT[t], 0.85)}"/></marker>`;
		return `<svg width="${L.W}" viewBox="0 0 ${L.W} ${L.H}" height="${L.H}" style="display:block" role="img" aria-label="Graph"><defs>${defs}</defs>${groups}${edges}${nodes}</svg>`;
	}
	// v3-core cgp/group.rs compose(own, scale, pattern): a Ring member's
	// pattern is 1, so its weight is scale + own.
	const compose = (own, scale, pattern) => scale * pattern + own;
	function edgeGroupsHtml(view, R, tick) {
		const gs = view.edgeGroups || [];
		if (!gs.length) return "";
		return `<div class="pv-h">${esc(view.edgeGroupsTitle || "Edge groups")} (${gs.length})</div>` + gs.map((g) => {
			const off = ((Math.round(g.offset || 0) % 8) + 8) % 8, cells = R && own(tick.rings, g.ring);
			const own8 = (c) => Number(own(g.own || {}, DN[c]) || 0);
			const present = g.members ? g.members.map((m) => DN.indexOf(m)) : [0, 1, 2, 3, 4, 5, 6, 7];
			let row = `<span>${esc(g.ring)}</span><span style="color:${MUTED}">${esc(g.kind)}</span><span>offset ${off}</span><span>scale=${Number(g.scale).toFixed(3)}</span><span style="color:${rgba(TINT.gm, 1)}">${present.length}/8 ${DN.map((_, c) => (present.includes(c) ? "●" : "○")).join("")}</span>`;
			if (g.gain) row += `<span style="color:#d8b4fe">gain=${esc(g.gain)}</span>`;
			if (g.isNew) row += `<span style="color:#fbbf24">new</span>`;
			if (g.note) row += `<span style="color:${MUTED}">${esc(g.note)}</span>`;
			if (cells) {
				const k = g.gain ? R.val[g.gain] || 0 : 1;
				const out = present.map((c) => (cells[c] ? `${g.kind}[${DN[(c + off) % 8]}] ${signed(compose(own8(c), Number(g.scale), 1) * cells[c] * k)}` : "")).filter(Boolean);
				row += `<span style="color:${VALUE}">→ ${esc(out.join(", ") || "nothing")}</span>`;
			}
			return `<div class="pv-rl">${row}</div>`;
		}).join("");
	}
	function gridHtml(grid, R) {
		if (!grid) return "";
		const val = (r) => (typeof r.key === "number" ? r.key : R ? R.val[r.key] : undefined);
		const cell = (x, color) => `<td style="color:${color}">${x}</td>`;
		const dot = cell("·", "#475569");
		let h = `<div class="pv-h">${esc(grid.title || "Weight grid")}</div><div class="pv-gb"><table class="pv-wg"><tr><th style="text-align:left;padding-left:6px;width:24%">input</th><th>value</th>${grid.cols.map((c) => `<th>${esc(c)}</th>`).join("")}</tr>`;
		grid.rows.forEach((r) => {
			const v = val(r);
			h += `<tr><td class="pv-l">${esc(r.label)}</td>${cell(R && v !== undefined ? f2(v) : "", VALUE)}${r.w.map((w) => (w == null ? dot : cell((w > 0 ? "+" : "") + w.toFixed(2), w > 0 ? "#6ee7b7" : w < 0 ? "#fda4af" : MUTED))).join("")}</tr>`;
		});
		if (R && grid.vote) {
			const sums = grid.cols.map((_, j) => grid.rows.reduce((s, r) => (r.w[j] == null ? s : (s || 0) + r.w[j] * (val(r) || 0)), null));
			const mx = Math.max(...sums.filter((s) => s !== null));
			h += `<tr><td class="pv-l" style="color:#fdba74">${esc(grid.vote)}</td><td></td>${sums.map((s) => (s === null ? dot : cell(f2(s), s === mx ? VALUE : MUTED))).join("")}</tr>`;
		}
		return h + "</table></div>";
	}
	function sceneHtml(G, R, tick) {
		const sc = tick.scene, note = tick.note ? esc(tick.note) + "<br>" : "";
		if (!sc) return note ? `<div class="pv-msg">${note}</div>` : "";
		const r = sc.r || 3, c = 16, W = c * (2 * r + 1), at = (x, y) => [(x + r) * c + c / 2, (y + r) * c + c / 2];
		const colors = { kin: "#34d399", stranger: "#f87171" };
		let s = `<rect width="${W}" height="${W}" rx="6" fill="#020617" stroke="#1e293b"/>`, dir = sc.arrow;
		for (let i = 1; i <= 2 * r; i++) s += `<path d="M${i * c} 0V${W}M0 ${i * c}H${W}" stroke="#0f172a"/>`;
		const [ox, oy] = at(0, 0);
		if (dir && typeof dir === "object") {
			let best = 0.004;
			const re = new RegExp(`${dir.kind}\\[(${DN.join("|")})\\]`);
			dir = null;
			G.nodes.forEach((n) => {
				const m = n.t === "out" && n.label.match(re);
				if (m && R.val[n.id] > best) (best = R.val[n.id], dir = m[1]);
			});
		}
		if (dir) {
			const i = DN.indexOf(dir), dx = [0, 1, 1, 1, 0, -1, -1, -1][i] * c * 0.62, dy = [-1, -1, 0, 1, 1, 1, 0, -1][i] * c * 0.62;
			s += `<path d="M${ox} ${oy}l${dx} ${dy}" stroke="#fb923c" stroke-width="2"/><circle cx="${ox + dx}" cy="${oy + dy}" r="2.5" fill="#fb923c"/>`;
		}
		(sc.creatures || []).forEach(([x, y, k]) => (s += `<circle cx="${at(x, y)[0]}" cy="${at(x, y)[1]}" r="5" fill="${own(colors, k) || MUTED}"/>`));
		s += `<circle cx="${ox}" cy="${oy}" r="5.5" fill="#e2e8f0"/>`;
		return `<div class="pv-see"><svg width="${W}" height="${W}" role="img" aria-label="What the creature sees">${s}</svg><div class="pv-msg">${note}<span style="color:${MUTED}">white = this creature · red = stranger · green = relative · gray = other${dir ? " · orange = chosen direction" : ""}</span></div></div>`;
	}

	const M = "font:11px ui-monospace,Menlo,monospace;";
	const CSS = `.pv-pb{display:flex;flex-wrap:wrap;gap:6px;margin:0 0 10px;align-items:center}.pv-pb button{font-size:13px}.pv-pb button[aria-pressed=true]{background:var(--bg-accent);color:var(--text-accent);border-color:var(--border-accent)}.pv-cap{font-size:13px;color:var(--text-secondary);line-height:1.5;margin:0 0 4px}.pv-ip{background:#0f172a;border-radius:12px;padding:10px 12px 12px}.pv-h{${M}font-weight:500;letter-spacing:.06em;text-transform:uppercase;color:${MUTED};margin:10px 0 6px}.pv-gb{background:#020617;border:1px solid #1e293b;border-radius:8px;padding:4px;overflow-x:auto}.pv-rl{${M}color:#cbd5e1;margin-top:3px;display:flex;gap:12px;flex-wrap:wrap}.pv-see{display:flex;gap:12px;align-items:center;margin-bottom:8px}.pv-msg{${M}font-size:12px;color:#cbd5e1;line-height:1.6}.pv-wg{border-collapse:collapse;width:100%;table-layout:fixed;${M}}.pv-wg th,.pv-wg td{padding:4px 2px;text-align:center;border-bottom:1px solid #0f172a}.pv-wg th{color:${MUTED};font-weight:500}.pv-wg .pv-l{text-align:left;color:#6ee7b7;padding-left:6px;overflow:hidden;white-space:nowrap}.pv-cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-top:10px}.pv-cards div{background:var(--surface-1);border-radius:var(--radius);padding:8px 12px}.pv-cards b{display:block;font-size:12px;font-weight:400;color:var(--text-secondary)}.pv-cards span{font-size:15px;font-weight:500;color:var(--text-primary)}`;
	let mounted = 0;

	function mount(target, spec) {
		const el = typeof target === "string" ? document.querySelector(target) : target;
		const uid = `pv${mounted++}`, state = { v: spec.startView || 0, t: spec.startTick === undefined ? -1 : spec.startTick };
		if (!document.getElementById("pv-css")) document.head.insertAdjacentHTML("beforeend", `<style id="pv-css">${CSS}</style>`);
		const bar = (k, items, cur, lead) => `<div class="pv-pb" data-k="${k}">${lead || ""}${items.map(([i, name]) => `<button data-i="${i}" aria-pressed="${i === cur}">${esc(name)}</button>`).join("")}</div>`;
		function draw() {
			const view = spec.views[state.v], ticks = view.ticks || [], tick = ticks[state.t], G = prepare(view);
			const d = depthOf(G), R = tick ? evaluate(G, tick) : null, L = layout(G, d, ticks.length > 0, view.sort);
			const count = (t) => G.nodes.filter((n) => n.t === t).length;
			let h = spec.views.length > 1 ? bar("v", spec.views.map((v, i) => [i, v.name]), state.v) : "";
			if (ticks.length) h += bar("t", [[-1, "Off"]].concat(ticks.map((t, i) => [i, t.name])), state.t, `<span class="pv-cap">Example tick:</span>`);
			if (view.caption) h += `<p class="pv-cap">${esc(view.caption)}</p>`;
			h += `<div class="pv-ip"><div class="pv-h" style="margin-top:0">${esc(tick ? `Example tick · ${tick.name}` : view.header || `Graph (${count("in")} in · ${count("cn")} compute · ${count("out")} out)`)}</div>`;
			if (tick) h += sceneHtml(G, R, tick);
			h += `<div class="pv-gb">${G.nodes.length ? graphSvg(G, L, R, uid, view.edgeLabels !== false) : `<div class="pv-msg" style="padding:16px;text-align:center">${esc(view.empty || "No graph edges.")}</div>`}</div>`;
			h += edgeGroupsHtml(view, R, tick) + gridHtml(view.grid, R) + "</div>";
			if (view.cards) h += `<div class="pv-cards">${view.cards.map((c) => `<div><b>${esc(c[0])}</b><span>${esc(c[1])}</span></div>`).join("")}</div>`;
			(view.notes || []).forEach((n) => (h += `<p class="pv-cap" style="margin-top:8px">${esc(n)}</p>`));
			el.innerHTML = h;
		}
		el.addEventListener("click", (e) => {
			const b = e.target.closest("button"), k = b && b.closest("[data-k]");
			if (!k) return;
			const bar = k.dataset.k, i = b.dataset.i;
			if (bar === "v") (state.v = +i, state.t = -1);
			else state.t = +i;
			draw();
			const again = el.querySelector(`[data-k="${bar}"] button[data-i="${i}"]`);
			if (again) again.focus();
		});
		draw();
	}

	root.PetriViz = { mount, CAT, TINT, kindLabel, kindCat, evalKind };
})(typeof window !== "undefined" ? window : globalThis);
