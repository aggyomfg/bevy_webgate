use crate::error::HttpErrorResponses;
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bevy_app::{App, Plugin};
use bevy_defer::AsyncWorld;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_log::error;
use std::{collections::HashSet, fs};

pub struct WebStaticAssetsPlugin;

impl Plugin for WebStaticAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WebStaticFileExtensions>();

        app.add_plugins(crate::error::HttpErrorPlugin);
    }
}

#[derive(Clone, Deref, DerefMut, Resource)]
pub struct WebStaticFileExtensions {
    extensions: HashSet<String>,
}

impl WebStaticFileExtensions {
    const DEFAULT_EXTENSIONS: [&'static str; 15] = [
        "css", "js", "png", "jpg", "jpeg", "gif", "svg", "ico", "woff", "woff2", "ttf", "eot",
        "pdf", "webp", "avif",
    ];
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_extensions<I, S>(extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            extensions: extensions
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }

    pub fn add_extension<S: Into<String>>(&mut self, extension: S) {
        self.extensions.insert(extension.into());
    }

    pub fn remove_extension(&mut self, extension: &str) {
        self.extensions.remove(extension);
    }

    #[must_use]
    pub fn contains(&self, extension: &str) -> bool {
        self.extensions.contains(extension)
    }

    pub fn clear(&mut self) {
        self.extensions.clear();
    }

    #[must_use]
    pub fn is_static_asset(file_path: &str) -> bool {
        let Some(extension) = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
        else {
            return false;
        };

        AsyncWorld
            .resource::<Self>()
            .get(|extensions| extensions.contains(extension))
            // Fall back to default extensions if the resource is not available
            .unwrap_or_else(|_| Self::DEFAULT_EXTENSIONS.contains(&extension))
    }
}

impl Default for WebStaticFileExtensions {
    fn default() -> Self {
        Self {
            extensions: Self::DEFAULT_EXTENSIONS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        }
    }
}

// Async is part of the public API so it can be awaited from handlers.
#[allow(clippy::unused_async)]
pub async fn serve_file(file_path: &str) -> Response {
    let safe_path = crate::utils::sanitize_path(file_path);

    let Ok(contents) = fs::read(&safe_path) else {
        bevy_log::info!("File not found: {}", safe_path);

        // Try to get custom 404 response
        return AsyncWorld
            .resource::<HttpErrorResponses>()
            .get(|responses| responses.create_response(StatusCode::NOT_FOUND))
            .unwrap_or_else(|_| {
                error!("Failed to create 404 response, using default");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service temporarily unavailable",
                )
                    .into_response()
            });
    };

    let mut headers = HeaderMap::new();

    let mime_type = mime_guess::from_path(&safe_path).first_or_octet_stream();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime_type.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );

    // Add cache control for static assets
    if WebStaticFileExtensions::is_static_asset(&safe_path) {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        );
    }

    (headers, contents).into_response()
}
