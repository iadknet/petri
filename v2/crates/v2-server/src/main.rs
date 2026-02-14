use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let bind = parse_bind(std::env::args().collect());
    if let Err(error) = v2_server::server::run(bind).await {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}

fn parse_bind(args: Vec<String>) -> SocketAddr {
    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--bind"
            && let Some(value) = iter.next()
        {
            if let Ok(addr) = value.parse::<SocketAddr>() {
                return addr;
            }
            eprintln!("invalid --bind value: {value}, falling back to default");
        }
    }
    SocketAddr::from(([127, 0, 0, 1], 4100))
}
