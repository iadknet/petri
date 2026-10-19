// Scratch probe behind Section 3 of the weight-decay research note
// (2026-10-19). Not part of the build: to re-run, append it to
// crates/v3-core/src/runtime/plasticity/tests.rs (it uses that module's
// `graph`, `genome`, `visit`, `weight`, `trace` and `reward` fixtures), run
//   cargo test -p v3-core --lib scratch_probe_weight_decay_random_walk -- --ignored --nocapture
// for arm A (zero-mean reward), and the same with PROBE_CORRELATED=1 set for
// arm B (reward follows the input 30% of the time; signed mean reported).
// Written against main at 98e3337a.

/// Zero-mean coincidental reward drives a single-component reward-modulated
/// weight on a random walk; a per-tick retention toward the birth weight
/// bounds it.
#[test]
#[ignore = "scratch probe for the weight-decay research note"]
fn scratch_probe_weight_decay_random_walk() {
    let correlated = std::env::var("PROBE_CORRELATED").is_ok();
    let mut lcg: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        lcg = lcg
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((lcg >> 40) as f32) / ((1u64 << 24) as f32)
    };
    let birth = 0.5f32;
    let checkpoints = [100usize, 1000, 5000, 20000];
    for eta in [0.005f32, 0.05] {
        for rho in [1.0f32, 0.999, 0.99, 0.9] {
            let seeds = 20;
            let mut mean_dev = vec![0.0f32; checkpoints.len()];
            let mut at_clamp = vec![0usize; checkpoints.len()];
            for _ in 0..seeds {
                // Classic rule, learning rate eta, trace_decay 0.9, clamp 2.0,
                // genomic weight 0.5, reward source EnergyDelta.
                let def = graph(HebbianRule::Classic, eta, 0.9);
                let mut state = GraphRuntimeState::for_genome(&genome(&def), Vec::new());
                let runtime = RuntimeConfig::default();
                for tick in 0..*checkpoints.last().unwrap() {
                    state.begin_tick(tick as u64);
                    // Decay at the start of the tick, before learning (Section 5
                    // places it in `begin_tick`); skipped before the first visit.
                    if rho < 1.0 && tick > 0 {
                        let w = weight(&state);
                        let t = trace(&state);
                        state.learned.set_slot(0, 0, birth + rho * (w - birth), t);
                    }
                    let input = next();
                    visit(&def, &mut state, input, &mut 100.0, &runtime);
                    let noise = if next() < 0.5 { -1.0 } else { 1.0 };
                    let signal = if correlated && next() < 0.3 {
                        if input > 0.5 { 1.0 } else { -1.0 }
                    } else {
                        noise
                    };
                    reward(&mut state, signal);
                    if let Some(k) = checkpoints.iter().position(|&c| c == tick + 1) {
                        let w = weight(&state);
                        let dev = if correlated { w - birth } else { (w - birth).abs() };
                        mean_dev[k] += dev / seeds as f32;
                        at_clamp[k] += usize::from(w.abs() >= 2.0 - 1e-6);
                    }
                }
            }
            eprintln!(
                "eta={eta} rho={rho}: mean deviation at {checkpoints:?} = {mean_dev:?}; at clamp (of {seeds}) = {at_clamp:?}"
            );
        }
    }
}
