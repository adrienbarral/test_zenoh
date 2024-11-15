use clap::Parser;
use zenoh::{pubsub::Publisher, qos::CongestionControl};
use zenoh_ext::z_serialize;
use log::{error, info};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // va créer des publisher entre "start" et "end"
    #[arg(short, long)]
    start: u16,
    #[arg(short, long)]
    end: u16,
}

#[tokio::main]
async fn main() -> Result<(), zenoh::Error> {
    env_logger::init();
    let config = zenoh::Config::from_env()?;
    let session = zenoh::open(config).await?;

    let args = Args::parse();
    let mut publishers: Vec<Publisher> = vec![];

    info!("Creating publishers from {} to {}", args.start, args.end);
    for i in args.start..args.end {
        publishers.push(session
            .declare_publisher(format!("key/expression/{}", i))
            .congestion_control(CongestionControl::Block)
            .await?);
    }

    info!("Publishing samples... (press Ctrl-C to stop)");
    tokio::spawn(async move {
        let mut i: u32 = 0;
        loop {
            for publisher in publishers.iter() {
                let serialized = z_serialize(&i);
                if let Err(e) = publisher.put(serialized).await {
                    error!("Error: {} when sending value", e);
                }
            }
            i += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            info!("Published: {}", i);
        }
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}
