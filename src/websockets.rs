use async_tungstenite::{WebSocketStream, accept_async};
use std::net::{ TcpListener, TcpStream };
use smol::Async;

use crossbeam_channel::Sender;

pub fn server_thread(
    tx: Sender<WebSocketStream<Async<TcpStream>>>
) {
    smol::block_on(async {
        let server = Async::<TcpListener>::bind(([127, 0, 0, 1], 9001))
            .unwrap();

        loop {
            let (stream, _) = server.accept()
                .await.unwrap();

            // TODO not sure if it's ok to ignore the error
            // but it'll do for now
            if let Ok(ws) = accept_async(stream).await {
                let _ = tx.send(ws);
            }
        }
    });
}
