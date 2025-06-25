# Multi-Port Support for bevy_webserver

The `bevy_webserver` library now supports running multiple web servers on different ports within a single Bevy application. This allows you to create different services (e.g., public API, admin interface, health checks) on separate ports with port-specific routing configurations.

## Features

- **Multiple Server Ports**: Run servers on different ports simultaneously
- **Port-Specific Routing**: Configure different routes for different ports
- **IP Address Binding**: Bind servers to specific IP addresses (localhost, all interfaces, etc.)
- **Backward Compatibility**: Existing single-port applications continue to work unchanged
- **Flexible Router Configuration**: Apply router configurations to specific ports

## Basic Usage

### Multiple Ports with Different Routes

```rust
use axum::{response::Html, routing::get};
use bevy::prelude::*;
use bevy_webserver::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        
        .port_route(8080, "/", get(|| async { Html("<h1>Public API</h1>") }))
        .port_route(8080, "/api/users", get(|| async { Html("User data") }))
        
        .port_route(8081, "/", get(|| async { Html("<h1>Admin Panel</h1>") }))
        .port_route(8081, "/admin/status", get(|| async { Html("Admin status") }))
        
        .port_route(8082, "/", get(|| async { Html("<h1>Health Service</h1>") }))
        .port_route(8082, "/health", get(|| async { Html("OK") }))
        
        .run();
}
```

### Different IP Addresses per Port

```rust
use axum::{response::Html, routing::get};
use bevy::prelude::*;
use bevy_webserver::prelude::*;
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Public API - localhost only
        .add_server(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
        // Admin interface - all interfaces (be careful in production!)
        .add_server(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8081)
        
        .port_route(8080, "/", get(|| async { Html("Public API") }))
        .port_route(8081, "/admin", get(|| async { Html("Admin Interface") }))
        
        .run();
}
```

## Backward Compatibility

Existing applications using the single-port API continue to work unchanged:

```rust
use axum::{response::Html, routing::get};
use bevy::prelude::*;
use bevy_webserver::RouterAppExt;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .route("/", get(|| async { Html("<h1>Hello World</h1>") }))
        .run();
}
```

This will create a server on the default port (8080) as before.

## Examples

The library includes several working examples:

- `examples/multi_ip_port.rs` - Different IP addresses per port
- `examples/single_port.rs` - Single port usage (backward compatibility)
- `examples/hello_world.rs` - Basic web server example

Run examples with:

```bash
cargo run --example multi_ip_port
cargo run --example dynamic_config
```
