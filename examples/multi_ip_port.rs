use axum::{response::Html, routing::get};
use bevy::prelude::*;
use bevy_webgate::prelude::*;
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_server(IpAddr::V4(Ipv4Addr::LOCALHOST), 3030);

    app.add_server(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 3031);

    app
        .port_route(3030, "/", get(|| async { 
            Html("<h1>Public API (localhost only)</h1><p>This server only accepts connections from localhost</p>") 
        }))
        .port_route(3030, "/api/users", get(|| async { 
            Html(r#"{"users": ["alice", "bob", "charlie"]}"#) 
        }));

    app
        .port_route(3031, "/", get(|| async { 
            Html("<h1>Admin Interface (all interfaces)</h1><p>⚠️ This server accepts connections from any IP</p>") 
        }))
        .port_route(3031, "/admin/status", get(|| async { 
            Html(r#"{"status": "ok", "uptime": "5m", "memory": "128MB"}"#) 
        }))
        .port_route(3031, "/admin/shutdown", get(|| async { 
            Html("<h2>🛑 Shutdown Endpoint</h2><p>This would shut down the server in a real app</p>") 
        }));

    app.run();
}
