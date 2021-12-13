use std::process::Stdio;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let routes = warp::path("uci")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| ws_uci_session(websocket))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}

async fn ws_uci_session(ws: WebSocket) {
    let (tx, mut rx) = ws.split();
    let mut engine_stdin = spawn_lila(tx).await.unwrap();

    while let Some(result) = rx.next().await {
        if let Ok(msg) = result {
            engine_stdin.write_all(msg.as_bytes()).await.unwrap();
            engine_stdin.write_all(b"\n").await.unwrap();
        }
    }

    // Quit the engine when the user leaves
    engine_stdin.write_all(b"quit\n").await.unwrap();
}

async fn spawn_lila(
    mut tx: SplitSink<WebSocket, Message>,
) -> Result<ChildStdin, Box<dyn std::error::Error>> {
    let mut engine = Command::new("./lc0/build/release/lc0")
        .arg("--weights=./maia-chess/maia_weights/maia-1500.pb.gz")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = engine
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let stdin = engine
        .stdin
        .take()
        .expect("child did not have to handle to stdin");

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async move {
        let status = engine
            .wait()
            .await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    let mut reader = BufReader::new(stdout).lines();

    tokio::spawn(async move {
        while let Some(line) = reader.next_line().await.unwrap() {
            tx.send(Message::text(line)).await.unwrap();
        }
    });

    Ok(stdin)
}
