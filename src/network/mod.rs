mod smol_executor;

use std::{net::TcpListener, sync::{Arc, Mutex}};

use crate::structs::Values;

use self::smol_executor::*;
use async_compat::*;
use futures_util::sink::SinkExt;
use smol::{prelude::*, Async};

use async_tungstenite::{
    tungstenite::{handshake::derive_accept_key, protocol::Role, Message}, 
    WebSocketStream
};

use hyper::upgrade::Upgraded;
use eyre::{Result, Error};
use hyper::{
    Server, service::{make_service_fn, service_fn}, 
    Request, Body, Response, StatusCode, 
    header::{
        HeaderValue, 
        CONNECTION, UPGRADE, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY}
};

pub struct Context {
    pub values: Mutex<Values>,
    pub clients: Mutex<Vec<WebSocketStream<Compat<Upgraded>>>>,
}

pub async fn handle_clients(ctx: Arc<Context>) {
    let mut clients = ctx.clients.lock().unwrap();
    clients.retain_mut(|websocket| {
        smol::block_on(async {
            let next_future = websocket.next();

            let msg_future = 
                smol::future::poll_once(next_future);

            let msg = match msg_future.await {
                Some(v) => {
                    match v {
                        Some(Ok(v)) => Some(v),
                        Some(Err(_)) => return false,
                        None => None,
                    }
                },
                None => None,
            };


            if let Some(Message::Close(_)) = msg {
                return false;
            };

            websocket.send(
                Message::Text(
                    "hii".to_string()
                    )
                ).await.unwrap();

            true
        })
    })
}

pub fn server_thread(ctx: Arc<Context>) {
    smol::block_on(async {
        let tcp = TcpListener::bind("127.0.0.1:9001").unwrap();
        let listener = Async::new(tcp)
            .unwrap();


        let server = Server::builder(SmolListener::new(&listener))
                .executor(SmolExecutor)
                .serve(make_service_fn(|_| {
                    let ctx = ctx.clone();
                    async { Ok::<_, Error>(service_fn( move |req| {
                        let ctx = ctx.clone();
                        serve(ctx, req)
                    })) }
                }));
        
        server.await.unwrap();
    })
}

async fn serve_ws(
    ctx: Arc<Context>, 
    mut req: Request<Body>
) -> Result<Response<Body>> {
    let headers = req.headers();
    let key = headers.get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));
    let ver = req.version();
    
    // It needs to be detached in order to upgrade properly work
    smol::spawn(async move {
        let upgraded = hyper::upgrade::on(&mut req).await
            .expect("upgraded error");

        let client = WebSocketStream::from_raw_socket(
            upgraded.compat(),
            Role::Server,
            None,
        ).await;

        let mut clients = ctx.clients.lock().unwrap();

        clients.insert(1, client);
    }).detach();
    
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    *res.version_mut() = ver;

    res.headers_mut().append(
        CONNECTION, HeaderValue::from_static("Upgrade")
    );

    res.headers_mut().append(
        UPGRADE, HeaderValue::from_static("websocket")
    );

    res.headers_mut()
        .append(SEC_WEBSOCKET_ACCEPT, derived.unwrap().parse().unwrap());

    Ok(res)
}

async fn serve_http(
    ctx: Arc<Context>, 
    mut req: Request<Body>
) -> Result<Response<Body>> {
    Ok(Response::new(Body::from("hello http request!")))
}

async fn serve(ctx: Arc<Context>, req: Request<Body>) -> Result<Response<Body>> {
    if req.uri() != "/ws" {
        return serve_http(ctx, req).await
    } else {
        return serve_ws(ctx, req).await
    }
}