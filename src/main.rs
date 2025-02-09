mod other;

use crate::other::{setup_router, Thing};
use anyhow::Result as AnyhowResult;
use async_io::Async;
use axum::routing::{get, MethodRouter};
use axum::Router;
use bevy::prelude::*;
use bevy::reflect::erased_serde::serialize;
use bevy::tasks::IoTaskPool;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncExtension, AsyncPlugin, AsyncWorld};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::{service, Method, Request, Response};
use smol_axum::TowerToHyperService;
use smol_hyper::rt::{FuturesIo, SmolTimer};
use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use bevy_easy_database::{AddDatabaseMapping, DatabaseIgnore, DatabasePlugin};

fn main() {
    println!("{}", serde_json::to_string(&Thing::x(1.0)).unwrap());
    App::new()
        .add_plugins(WevyServerPlugins)
        .add_plugins(DatabasePlugin)
        .insert_resource(RouterWrapper(setup_router()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gizmo)
        .add_database_mapping::<Transform>()
        .run();
    println!("Hello, world!");
}
fn draw_gizmo(mut gimzos: Gizmos, transforms: Query<&mut Transform, Without<Camera>>) {
    for t in transforms.iter() {
        gimzos.sphere(t.translation, 5.0, Color::srgb(1.0, 0.0, 0.0));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), DatabaseIgnore));
}

async fn owo_get() -> String {
    AsyncWorld
        .query_single::<&Transform>()
        .get_mut(|q| -> String { serde_json::to_string(q).unwrap() })
        .unwrap()
}

pub struct WevyServerPlugins;

impl Plugin for WevyServerPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, AsyncPlugin::default_settings()));
        app.add_systems(Startup, start_server);
    }
}

fn start_server(world: &mut World) {
    let address = IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap());
    let port = 25569;
    world.spawn_task(async move {
        server_main(address, port).await.unwrap();
        Ok(())
    });
}

async fn server_main(address: IpAddr, port: u16) -> AnyhowResult<()> {
    listen(Async::<TcpListener>::bind((address, port))?).await
}
async fn listen(listener: Async<TcpListener>) -> AnyhowResult<()> {
    let router_wrapper: RouterWrapper = AsyncWorld
        .run(|world| -> RouterWrapper { world.remove_resource::<RouterWrapper>().unwrap() });
    let router = router_wrapper.0;
    let service = router.into_service();
    let service = TowerToHyperService { service };
    loop {
        let service = service.clone();
        let (client, _) = listener.accept().await?;
        AsyncWorld
            .spawn_task(async {
                http1::Builder::new()
                    .timer(SmolTimer::new())
                    .serve_connection(FuturesIo::new(client), service)
                    .await
                    .unwrap();
            })
            .detach();
        AsyncWorld.yield_now().await;
    }
}

#[derive(Resource)]
pub struct RouterWrapper(pub Router);
/*
#[derive(Resource)]
pub struct ServeMethods(pub HashMap<Method, HashMap<String, Box<dyn Handler<Future=Pin<Box<dyn Future<Output = Bytes> + Send + 'static>>>>>>);

async fn process_request(request: Request<Incoming>) -> AnyhowResult<Response<Full<Bytes>>> {
    use http_body_util::BodyExt;

    let path = request.uri().path().to_string();
    match request.method() {
        &Method::GET => {
            let bytes = request.into_body().collect().await.unwrap().to_bytes();
            let awa = AsyncWorld.resource_scope(|serve_methods: Mut<ServeMethods>| {
                serve_methods.0.get_mut(&Method::GET).unwrap().insert(String::default(), i_am_some_func);
                let serve_methods = serve_methods.0.get(&Method::GET).unwrap();
                serve_methods.get(&path).unwrap()
            });
            let response = AsyncWorld.spawn_task(awa(bytes)).await;
        }
        _ => {
            todo!()
        }
    }
    todo!()
}

async fn i_am_some_func(bytes: Bytes) -> Bytes {

    todo!()
}

pub trait Handler: Clone + Send + Sync + Sized + 'static {
    /// The type of future calling this handler returns.
    type Future: Future<Output = Bytes> + Send + 'static;

    /// Call the handler with the given request.
    fn call(self, req: Bytes) -> Self::Future;
}

impl<F, Fut, Res> Handler for F
where
    F: FnOnce(Bytes) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Res: Into<Bytes>,
{
    type Future = Pin<Box<dyn Future<Output = Bytes> + Send + 'static>>;

    fn call(self, req: Bytes) -> Self::Future {
        Box::pin(async move { self(req).await.into_response() })
    }
}
*/
