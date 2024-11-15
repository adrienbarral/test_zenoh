use std::sync::{atomic::{AtomicU16, Ordering}, Arc};

use clap::Parser;
use tokio::{signal::unix::{signal, SignalKind}, task::JoinSet};
use zenoh_ext::z_deserialize;
use log::{info, warn};

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
    
    let miss = Arc::new(AtomicU16::new(0));
    let hit = Arc::new(AtomicU16::new(0));

    let config = zenoh::Config::from_env()?;
    let session = zenoh::open(config).await.unwrap();
    let args = Args::parse();
    for i in args.start..args.end {
        let first_subscriber = i == args.start;
        let subscriber = session
            .declare_subscriber(format!("key/expression/{}", i))
            .with(flume::bounded(32)).await?;

        let hit_c = Arc::clone(&hit);
        let miss_c = Arc::clone(&miss);

        tokio::spawn(async move {
            let mut last_recveid: Option<u32> = None;
            while let Ok(sample) = subscriber.recv_async().await {
                let sample: u32 = z_deserialize(&sample.payload()).unwrap();
                if let Some(last_sample) = last_recveid {
                    if sample != last_sample + 1 {
                        warn!("Error: expected {} but got {}", last_sample + 1, sample);
                        miss_c.fetch_add(1, Ordering::Relaxed);
                    }else {
                        hit_c.fetch_add(1, Ordering::Relaxed);
                    }
                }
                last_recveid = Some(sample);
                if sample % 10 == 0 && first_subscriber{
                    info!("Received: {}", sample);
                }
            }
        });
    }

    info!("Waiting for samples... (press Ctrl-C to stop)");
    
    let mut stream = signal(SignalKind::terminate())?;
    stream.recv().await;
    session.close().await?;

    let all_hit = hit.load(Ordering::Relaxed);
    let all_miss = miss.load(Ordering::Relaxed);

    info!("Total hits: {}", all_hit);
    info!("Total miss: {}", all_miss);
    info!("Loss ratio: {}", all_miss as f32 / (all_hit + all_miss) as f32);

    Ok(())
}
