use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{extract::Path, Json, Router};
use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncWorld};
use serde::{Deserialize, Serialize};

// Spawn a new transform
async fn spawn_transform(
    Json(transform): Json<Transform>,
    //s: String,
) -> Html<String> {
    /*println!("{:#?}", s);
    println!("{:#?}", serde_json::to_string(&Transform::default()));
    return Html("".to_string());*/
    AsyncWorld.apply_command(move |mut world: &mut World| {
        println!("spawning: {:?}", transform);
        world.spawn(transform);
    });

    // Return the updated list of transforms
    get_transforms().await
}

// Get all transforms
async fn get_transforms() -> Html<String> {
    Html(
        AsyncWorld
            .query_filtered::<(Entity, &Transform), Without<Camera>>()
            .get_mut(|mut transforms| {
                transforms
                    .iter()
                    .map(|(entity, transform)| {
                        format!(
                            r#"
                <div class="card">
                    <div class="transform-grid">
                        <div>
                            <div class="vector-label">Translation</div>
                            <input type="number" name="x" value="{}" step="10"
                                   hx-put="/transform/{}/translation"
                                   hx-headers='{{"Content-Type": "application/json"}}'
                                   parse-types="true"
                                   hx-ext="json-enc-custom"
                                   hx-trigger="change">
                            <input type="number" name="y" value="{}" step="10"
                                   hx-put="/transform/{}/translation"
                                   hx-headers='{{"Content-Type": "application/json"}}'
                                   parse-types="true"
                                   hx-ext="json-enc-custom"
                                   hx-trigger="change">
                            <input type="number" name="z" value="{}" step="10"
                                   hx-put="/transform/{}/translation"
                                   hx-headers='{{"Content-Type": "application/json"}}'
                                   parse-types="true"
                                   hx-ext="json-enc-custom"
                                   hx-trigger="change">
                        </div>
                    </div>
                    <button hx-delete="/transform/{}"
                            hx-target="closest .card"
                            hx-swap="outerHTML">Delete</button>
                </div>
            "#,
                            transform.translation.x,
                            serde_json::to_string(&entity).unwrap(),
                            transform.translation.y,
                            serde_json::to_string(&entity).unwrap(),
                            transform.translation.z,
                            serde_json::to_string(&entity).unwrap(),
                            serde_json::to_string(&entity).unwrap()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap(),
    )
}

use serde_this_or_that::as_f64;
#[derive(Serialize, Deserialize)]
pub enum Thing {
    #[serde(deserialize_with = "as_f64")]
    x(f64),
    #[serde(deserialize_with = "as_f64")]
    y(f64),
    #[serde(deserialize_with = "as_f64")]
    z(f64),
}
// Update transform translation
async fn update_translation(
    Path(entity): Path<Entity>,
    Json(translation): Json<Thing>,
    //s: String,
) {
    AsyncWorld
        .query::<&mut Transform>()
        .get_mut(move |mut transforms| {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                match translation {
                    Thing::x(x) => transform.translation.x = x as f32,
                    Thing::y(y) => transform.translation.y = y as f32,
                    Thing::z(z) => transform.translation.z = z as f32,
                }
                //transform.translation = translation;
            }
        })
        .unwrap();
}

// Delete transform
async fn delete_transform(Path(entity): Path<Entity>) {
    AsyncWorld.apply_command(move |mut world: &mut World| {
        world.despawn(entity);
    });
}

async fn root() -> Html<String> {
    Html(include_str!("./other.html").to_string())
}

// Add these routes to your router
pub fn setup_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/spawn", post(spawn_transform))
        .route("/transforms", get(get_transforms))
        .route("/transform/{entity}/translation", put(update_translation))
        .route("/transform/{entity}", delete(delete_transform))
}
