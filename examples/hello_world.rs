fn main() {
    use bevy_webserver::prelude::*;
    bevy::prelude::App::new()
        .add_plugins((
            bevy::prelude::MinimalPlugins,
            bevy_webserver::BevyWebServerPlugin,
        ))
        .route("/hello_world", axum::routing::get(hello_world))
        .run();
}

async fn hello_world() -> axum::response::Html<String> {
    axum::response::Html("<p> hello world! </p>".to_string())
}
