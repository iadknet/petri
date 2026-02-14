fn main() {
    let mut api = v2_server::api::SimulationApi::new();
    let startup = api.startup(v2_server::api::StartupRequest::default());
    println!(
        "{} server contract surface ready (protocol {}, state {}, tick {})",
        v2_core::runtime_name(),
        startup.protocol_version,
        startup.state,
        startup.tick
    );
}
