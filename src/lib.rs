use bevy_app::{App, Plugin, Startup, Update};
use bevy_defer::AsyncPlugin;
use std::net::{IpAddr, Ipv4Addr};

mod app_ext;
mod error;
mod server;
mod static_assets;

pub mod prelude;
pub mod utils;

#[cfg(test)]
mod tests;

pub const DEFAULT_PORT: WebPort = 8080;
pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub use app_ext::*;
pub use error::*;
pub use server::{WebPort, WebServer, WebServerConfig, WebServerManager};
pub use static_assets::*;

pub struct BevyWebServerPlugin;

impl Plugin for BevyWebServerPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AsyncPlugin>() {
            app.add_plugins(AsyncPlugin::default_settings());
        }

        app.add_plugins(WebStaticAssetsPlugin);

        let world = app.world_mut();

        world.init_resource::<WebServerManager>();

        if let Some(single_config) = world.get_resource::<WebServerConfig>() {
            let legacy_config = WebServerManager::from(single_config.clone());
            world.insert_resource(legacy_config);
        }

        app.add_systems(Startup, WebServerManager::changed)
            .add_systems(
                Update,
                (
                    WebServerManager::changed,
                    WebServerManager::cleanup_finished_tasks,
                    WebServerManager::check_retry_servers,
                ),
            );
    }
}
