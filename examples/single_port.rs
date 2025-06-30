use axum::{response::Html, routing::get};
use bevy::prelude::*;
use bevy_webgate::{RouterAppExt, WebServerConfig};
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .insert_resource(WebServerConfig {
            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 8080,
        })
        .route(
            "/",
            get(|| async { Html("<h1>Backward Compatibility Test</h1>") }),
        )
        .route(
            "/test",
            get(|| async { Html("<h2>This is a test route</h2>") }),
        )
        .run();
}
