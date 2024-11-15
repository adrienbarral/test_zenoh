use futures::future::join_all;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];

    for i in 0..100 {
        let start = i * 10;
        let end = (i + 1) * 10;
        let mut child1 = Command::new("target/debug/publisher")
            .arg("-s")
            .arg(start.to_string())
            .arg("-e")
            .arg(end.to_string())
            .spawn()
            .expect("failed to spawn");

        handles.push(tokio::spawn(async move {
            child1.wait().await.expect("child process 1 failed");
        }));
    }

    // await all childs have finished :
    join_all(handles).await;

    Ok(())
}
