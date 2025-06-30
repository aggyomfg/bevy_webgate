use axum::routing::get;
use bevy::prelude::*;
use bevy_webgate::prelude::*;
use std::time::Duration;

#[derive(Resource)]
struct ShutdownTimer {
    timer: Timer,
    shutdown_initiated: bool,
}

impl Default for ShutdownTimer {
    fn default() -> Self {
        Self {
            // Wait 10 seconds before initiating graceful shutdown
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            shutdown_initiated: false,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_webgate::BevyWebServerPlugin))
        .init_resource::<ShutdownTimer>()
        .route("/", get(hello_handler))
        .route("/slow", get(slow_handler))
        .route("/status", get(status_handler))
        .add_systems(Startup, print_startup_info)
        .add_systems(Update, graceful_shutdown_demo)
        .run();
}

fn print_startup_info() {
    println!("Graceful Shutdown Demo");
    println!("1. Server runs for 10 seconds");
    println!("2. Graceful shutdown initiated with 10 second timeout");
    println!("3. Library handles the shutdown process automatically:");
    println!("   - Stops accepting new connections");
    println!("   - Waits for existing connections to complete");
    println!("   - Forces shutdown after timeout if needed");
    println!();
    println!("Try making requests to /slow during shutdown to see graceful handling!");
    println!("Watch the logs to see the library-level timeout handling in action!");
}

async fn hello_handler() -> axum::response::Html<&'static str> {
    axum::response::Html("<h1>Hello World!</h1><p>This is a quick response.</p>")
}

async fn slow_handler() -> axum::response::Html<&'static str> {
    // Simulate a slow operation using std::thread::sleep (blocking but simple)
    println!("Slow request started, will take 5 seconds...");

    for i in 1..=5 {
        std::thread::sleep(Duration::from_secs(1));
        println!("Slow request progress: {}s", i);
    }

    println!("Slow request completed!");
    axum::response::Html("<h1>Slow Response</h1><p>This response took 5 seconds to generate.</p>")
}

async fn status_handler() -> axum::response::Html<String> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    axum::response::Html(format!(
        "<h1>Server Status</h1>\
         <p>Server is running</p>\
         <p>Current time: {}</p>\
         <p>Try the <a href=\"/slow\">/slow</a> endpoint to test graceful shutdown behavior.</p>",
        uptime
    ))
}

fn graceful_shutdown_demo(
    time: Res<Time>,
    mut shutdown_timer: ResMut<ShutdownTimer>,
    mut manager: ResMut<WebServerManager>,
    mut commands: Commands,
) {
    shutdown_timer.timer.tick(time.delta());
    if shutdown_timer.timer.just_finished() && !shutdown_timer.shutdown_initiated {
        shutdown_timer.shutdown_initiated = true;
        manager.graceful_shutdown_with_timeout(&8080, Duration::from_secs(10), &mut commands);
        info!("Graceful shutdown with 10s timeout initiated");
    }
}
