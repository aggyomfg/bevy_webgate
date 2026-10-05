use super::*;
use async_io::Async;
use axum::{response::Html, routing::get, Router};
use bevy::prelude::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

#[test]
fn test_web_servers_basic_operations() {
    let mut servers = WebServerManager::default();
    assert_eq!(servers.len(), 0);
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            18080,
            Router::new(),
        ))
        .unwrap();
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            18081,
            Router::new(),
        ))
        .unwrap();
    servers
        .add_server(WebServer::new(
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            18082,
            Router::new(),
        ))
        .unwrap();

    assert_eq!(servers.len(), 3);

    let server_8080 = servers.get_server(18080).unwrap();
    assert_eq!(server_8080.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(server_8080.port(), 18080);

    let server_8081 = servers.get_server(18081).unwrap();
    assert_eq!(server_8081.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(server_8081.port(), 18081);

    let server_8082 = servers.get_server(18082).unwrap();
    assert_eq!(server_8082.ip(), IpAddr::V6(Ipv6Addr::LOCALHOST));
    assert_eq!(server_8082.port(), 18082);

    assert!(servers.get_server(9999).is_none());
}

#[test]
fn test_web_servers_insert_and_update() {
    let mut servers = WebServerManager::default();

    let router1 = Router::new().route("/api", get(|| async { "api v1" }));
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            19080,
            router1,
        ))
        .unwrap();

    assert_eq!(servers.len(), 1);
    let server = servers.get_server(19080).unwrap();
    assert_eq!(server.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));

    // Update existing server router (should replace)
    let router2 = Router::new().route("/api", get(|| async { "api v2" }));
    servers.set_router(&19080, router2);

    assert_eq!(servers.len(), 1); // Still only one entry

    let custom_router = Router::new().route("/custom", get(|| async { "custom" }));
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            19081,
            custom_router,
        ))
        .unwrap();

    let server = servers.get_server(19081).unwrap();
    assert_eq!(server.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
}

#[test]
fn test_app_extensions() {
    let mut app = create_test_app();

    app.add_server(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 20081);

    assert_eq!(app.server_count(), 1);

    let running_servers = app.running_servers();
    assert_eq!(running_servers.len(), 1);
    assert_eq!(
        running_servers[0],
        (20081, IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    );

    let ports = app.routed_ports();
    assert_eq!(ports.len(), 1);
    assert!(ports.contains(&20081));

    // Test routing - should create port if it doesn't exist
    app.port_route(20082, "/test", get(|| async { Html("test") }));
    assert_eq!(app.server_count(), 2);
    let ports = app.routed_ports();
    assert!(ports.contains(&20082));

    // Test adding route to existing port
    app.port_route(20082, "/test2", get(|| async { Html("test2") }));
    assert_eq!(app.server_count(), 2);

    // Test route method (should use default port)
    app.route("/default", get(|| async { Html("default") }));
    assert_eq!(app.server_count(), 3);
}

#[test]
fn test_server_mutable_operations() {
    let mut servers = WebServerManager::default(); // Setup initial servers
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            21080,
            Router::new().route("/api", get(|| async { "api" })),
        ))
        .unwrap();
    servers
        .add_server(WebServer::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            21081,
            Router::new().route("/admin", get(|| async { "admin" })),
        ))
        .unwrap();

    // Test mutable access and modification
    if let Some(server) = servers.get_server_mut(21080) {
        *server.router_mut() = server
            .router()
            .clone()
            .route("/api/v2", get(|| async { "api v2" }));
    }

    assert!(servers.get_server_mut(21080).is_some());
    assert!(servers.get_server_mut(9999).is_none());

    let ports: Vec<u16> = servers.ports();
    assert_eq!(ports.len(), 2);
    assert!(ports.contains(&21080));
    assert!(ports.contains(&21081));
}

#[test]
fn test_ipv6_support() {
    let mut app = create_test_app();

    let localhost_v6 = "::1".parse::<IpAddr>().unwrap();
    let localhost_v4 = "127.0.0.1".parse::<IpAddr>().unwrap();
    let unspecified_v4 = "0.0.0.0".parse::<IpAddr>().unwrap();
    app.add_server(localhost_v6, 22080);
    app.add_server(localhost_v4, 22081);
    app.add_server(unspecified_v4, 22082);

    let servers = app.world().get_resource::<WebServerManager>().unwrap();

    assert_eq!(servers.get_server(22080).unwrap().ip(), localhost_v6);
    assert_eq!(servers.get_server(22081).unwrap().ip(), localhost_v4);
    assert_eq!(servers.get_server(22082).unwrap().ip(), unspecified_v4);
}

#[test]
fn test_backward_compatibility() {
    let mut app = create_test_app();

    // Test old single config format
    app.world_mut().insert_resource(WebServerConfig {
        ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 8080,
    });

    app.add_plugins(BevyWebServerPlugin);

    let servers = app.world().get_resource::<WebServerManager>().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.has_server(&8080));

    let server = servers.get_server(8080).unwrap();
    assert_eq!(server.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));

    // Test that routes can be added to the legacy port
    app.route("/old", get(|| async { Html("old config") }));

    // Should still have only one server (the default route creates the default server)
    let servers = app.world().get_resource::<WebServerManager>().unwrap();
    assert!(!servers.is_empty());
}

#[test]
fn test_multiple_ports_same_ip() {
    let mut servers = WebServerManager::default();
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST); // Should be able to use same IP on different ports
    servers
        .add_server(WebServer::new(ip, 23080, Router::new()))
        .unwrap();
    servers
        .add_server(WebServer::new(ip, 23081, Router::new()))
        .unwrap();
    servers
        .add_server(WebServer::new(ip, 23082, Router::new()))
        .unwrap();

    assert_eq!(servers.len(), 3);
    assert_eq!(servers.get_server(23080).unwrap().ip(), ip);
    assert_eq!(servers.get_server(23081).unwrap().ip(), ip);
    assert_eq!(servers.get_server(23082).unwrap().ip(), ip);
}

#[test]
fn test_port_collision_handling() {
    let mut servers = WebServerManager::default();
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    servers
        .add_server(WebServer::new(
            Ipv4Addr::LOCALHOST.into(),
            port,
            Router::new(),
        ))
        .unwrap();

    let original_status = servers.get_server(port).unwrap().status();
    let original_error = servers.server_error(&port).unwrap().to_string();
    let result = servers.add_server(WebServer::new(
        Ipv4Addr::UNSPECIFIED.into(),
        port,
        Router::new(),
    ));

    assert!(matches!(
        result,
        Err(WebServerError::ServerAlreadyRunning { port: error_port }) if error_port == port
    ));
    assert_eq!(servers.len(), 1);
    assert_eq!(
        servers.get_server(port).unwrap().ip(),
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    );
    assert_eq!(servers.get_server(port).unwrap().status(), original_status);
    assert_eq!(servers.server_error(&port), Some(original_error.as_str()));
}

#[test]
fn test_port_bind_failure_schedules_retry() {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let error = WebServerManager::test_bind(ip, port).unwrap_err();
    assert!(matches!(
        error,
        WebServerError::BindFailed { ip: error_ip, port: error_port, source }
            if error_ip == ip
                && error_port == port
                && source.kind() == std::io::ErrorKind::AddrInUse
    ));

    let mut servers = WebServerManager::default();
    servers
        .add_server(WebServer::new(ip, port, Router::new()))
        .unwrap();

    let server = servers.get_server(port).unwrap();
    assert_eq!(server.status(), crate::server::ServerStatus::Retrying);
    assert!(server.last_error().is_some());
    assert!(!server.should_retry());
    assert!(!servers.server_failed(&port));

    drop(listener);
    assert!(WebServerManager::test_bind(ip, port).is_ok());
}

#[test]
fn test_graceful_shutdown_timeout_result() {
    let mut server = WebServer::new(Ipv4Addr::LOCALHOST.into(), 0, Router::new());
    server.set_status(crate::server::ServerStatus::Running);

    let completed =
        async_io::block_on(server.graceful_shutdown_with_timeout(std::time::Duration::ZERO));
    assert!(completed);
    assert_eq!(server.status(), crate::server::ServerStatus::Stopped);
    assert_eq!(server.count_active_connections(), 0);

    server.set_status(crate::server::ServerStatus::Running);
    let guard = server.new_connection();
    server.graceful_shutdown();
    assert!(server.shutdown_requested());
    assert!(!server.status().can_start());

    let completed =
        async_io::block_on(server.graceful_shutdown_with_timeout(std::time::Duration::ZERO));
    assert!(!completed);
    assert_eq!(server.status(), crate::server::ServerStatus::Stopped);
    assert_eq!(server.count_active_connections(), 1);

    drop(guard);
    assert_eq!(server.count_active_connections(), 0);
}

#[test]
fn test_web_servers_default_behavior() {
    let servers = WebServerManager::default();
    assert_eq!(servers.len(), 0);

    let count = servers.iter().count();
    assert_eq!(count, 0);
}

#[test]
fn test_error_cases() {
    let mut app = create_test_app();

    app.add_server(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let servers = app.world().get_resource::<WebServerManager>().unwrap();
    assert!(servers.has_server(&0));

    app.port_route(u16::MAX, "/", get(|| async { Html("max port") }));
    let servers = app.world().get_resource::<WebServerManager>().unwrap();
    assert!(servers.has_server(&u16::MAX));
}

#[test]
fn test_utility_methods_usage() {
    let mut app = create_test_app();

    assert_eq!(app.server_count(), 0);
    assert_eq!(app.routed_ports(), [] as [u16; 0]);
    app.add_server(IpAddr::V4(Ipv4Addr::LOCALHOST), 25080);
    app.add_server(IpAddr::V4(Ipv4Addr::LOCALHOST), 25081);
    app.add_server(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 25082);

    assert_eq!(app.server_count(), 3);

    let running_servers = app.running_servers();
    assert_eq!(running_servers.len(), 3);

    let server_map: std::collections::HashMap<u16, IpAddr> = running_servers.into_iter().collect();
    assert_eq!(
        server_map.get(&25080),
        Some(&IpAddr::V4(Ipv4Addr::LOCALHOST))
    );
    assert_eq!(
        server_map.get(&25081),
        Some(&IpAddr::V4(Ipv4Addr::LOCALHOST))
    );
    assert_eq!(
        server_map.get(&25082),
        Some(&IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    );

    let ports = app.routed_ports();
    assert_eq!(ports.len(), 3);
    assert!(ports.contains(&25080));
    assert!(ports.contains(&25081));
    assert!(ports.contains(&25082));
}

#[test]
fn test_add_server_port_convenience() {
    let mut app = create_test_app();

    app.port_route(26080, "/api", get(|| async { Html("API") }))
        .port_route(26081, "/admin", get(|| async { Html("Admin") }))
        .port_route(26082, "/health", get(|| async { Html("Health") }));

    let servers = app.world().get_resource::<WebServerManager>().unwrap();
    assert_eq!(servers.get_server(26080).unwrap().ip(), DEFAULT_IP);
    assert_eq!(servers.get_server(26081).unwrap().ip(), DEFAULT_IP);
    assert_eq!(servers.get_server(26082).unwrap().ip(), DEFAULT_IP);

    assert_eq!(app.server_count(), 3);
}

#[test]
fn test_exact_bind_replication() {
    let ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    let port = 17080;

    let test_bind_result = crate::server::WebServerManager::test_bind(ip, port);
    assert!(
        test_bind_result.is_ok(),
        "WebServerManager::test_bind should succeed for available port"
    );

    let async_bind_result = Async::<std::net::TcpListener>::bind((ip, port));
    assert!(
        async_bind_result.is_ok(),
        "Direct async bind should succeed for available port"
    );
}
