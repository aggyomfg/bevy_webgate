use async_io::Async;
use axum::routing::post;
use axum::{
    extract::State,
    response::Html,
    routing::{delete, get, put},
    Json,
};
use bevy::ecs::component::ComponentInfo;
use bevy::ecs::entity::Entities;
use bevy::reflect::{EnumInfo, StructInfo, TupleStructInfo, TypeInfo, TypeRegistry};
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_defer::AsyncWorld;
use bevy_webserver::{BevyWebServerPlugin, RouterAppExt};
use maud::{html, Markup, PreEscaped};

pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntity>()
            .register_type::<SelectedEntity>()
            .add_systems(PostUpdate, reset_selected_entity_if_entity_despawned);
    }
}

/// The currently selected entity in the scene.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

/// System to reset [`SelectedEntity`] when the entity is despawned.
pub fn reset_selected_entity_if_entity_despawned(
    mut selected_entity: ResMut<SelectedEntity>,
    entities: &Entities,
) {
    if let Some(e) = selected_entity.0 {
        if !entities.contains(e) {
            selected_entity.0 = None;
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WebInspectorPlugin, BevyWebServerPlugin))
        .insert_resource(SelectedEntity::default())
        .run();
}

pub struct WebInspectorPlugin;

impl Plugin for WebInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.route("/", get(render_layout))
            .route("/inspector", get(render_inspector))
            .route("/component/{entity}/{component}", put(update_component))
            .route("/component/{entity}", delete(delete_component))
            .route("/entities", get(render_entity_list))
            .route("/entities/select/{entity}", post(select_entity));
    }
}

// State structure to hold component values
#[derive(serde::Serialize, serde::Deserialize)]
struct ComponentValue {
    value: serde_json::Value,
}

async fn render_layout() -> Html<String> {
    Html(
        html! {
            html {
                head {
                    title { "Bevy Web Inspector" }
                    script src="https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.10/htmx.min.js" {}
                    script src="https://cdn.jsdelivr.net/gh/Emtyloc/json-enc-custom@main/json-enc-custom.js" {}
                }
                body class="m-0 p-0" {
                    div class="flex h-screen bg-zinc-900" {
                        // Entity list panel
                        div class="w-80 border-r border-zinc-700"
                            hx-get="/entities"
                            hx-trigger="load"
                            hx-swap="innerHTML" {}
                        // Inspector panel
                        div id="inspector"
                            class="flex-grow"
                            hx-get="/inspector"
                            hx-trigger="load"
                            hx-swap="innerHTML" {}
                    }
                }
            }
        }
            .into_string(),
    )
}
async fn render_entity_list() -> Html<String> {
    AsyncWorld.run(|world| -> Html<String> {
        let selected = world.resource::<SelectedEntity>().0;
        let markup = html! {
            div class="entity-list p-4 bg-zinc-800" {
                h2 class="text-lg font-medium mb-4 text-white" { "Entities" }

                div class="space-y-2" {
                    @for (entity, name) in get_named_entities(world) {
                        form class="entity-card bg-zinc-700 p-3 rounded"
                             hx-post=(format!("/entities/select/{}", entity.index()))
                             hx-target="#inspector"
                             hx-swap="innerHTML" {

                            // Entity info section
                            div class="flex justify-between items-center mb-2" {
                                // Left side: ID and name
                                div {
                                    span class="text-sm font-mono text-zinc-400" {
                                        "#" (entity.index())
                                    }
                                    @if let Some(name) = &name {
                                        span class="ml-2 text-sm text-white" {
                                            (name)
                                        }
                                    }
                                }

                                // Right side: component count
                                span class="text-xs text-zinc-400" {
                                    (get_component_count(world, entity)) " components"
                                }
                            }

                            button type="submit"
                                    class=(format!("px-3 py-1 rounded text-sm transition-colors {}",
                                        if Some(entity) == selected {
                                            "bg-emerald-600 hover:bg-emerald-500 text-white"
                                        } else {
                                            "bg-zinc-600 hover:bg-zinc-500 text-white"
                                        }
                                    )) {
                                @if Some(entity) == selected {
                                    "Selected"
                                } @else {
                                    "Select"
                                }
                            }
                        }
                    }
                }
            }
        };
        Html(markup.into_string())
    })
}

// Helper function to get entities with their names
fn get_named_entities(world: &mut World) -> Vec<(Entity, Option<String>)> {
    let mut entities = Vec::new();

    // Get the type registry to inspect components
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // Query for all entities that optionally have a Name component
    let mut query = world.query::<(Entity, Option<&Name>)>();
    for (entity, name) in query.iter(world) {
        let name = name.map(|name| name.as_str().to_string());

        // Only include entities that have at least one reflected component
        if world
            .inspect_entity(entity)
            .filter(|info| {
                let type_register = type_registry.clone();
                let type_register = type_register.read();
                info.type_id()
                    .and_then(|id| type_register.get_type_info(id))
                    .is_some()
            })
            .next()
            .is_some()
        {
            entities.push((entity, name));
        }
    }

    // Sort first by presence of name, then by entity ID for stable ordering
    entities.sort_by(|(entity_a, name_a), (entity_b, name_b)| {
        name_a
            .is_some()
            .cmp(&name_b.is_some())
            .reverse()
            .then_with(|| entity_a.index().cmp(&entity_b.index()))
    });

    entities
}

// Helper function to count components on an entity
fn get_component_count(world: &World, entity: Entity) -> usize {
    let type_registry = world.resource::<AppTypeRegistry>().read();

    // Only count reflected components
    world
        .inspect_entity(entity)
        .filter(|info| {
            info.type_id()
                .and_then(|id| type_registry.get_type_info(id))
                .is_some()
        })
        .count()
}

async fn select_entity(
    axum::extract::Path(entity_index): axum::extract::Path<u32>,
) -> Html<String> {
    AsyncWorld.run(|world| {
        // Create entity from index and update selected entity
        let entity = Entity::from_raw(entity_index);
        if world.get_entity(entity).is_ok() {
            world.resource_mut::<SelectedEntity>().0 = Some(entity);
        }
    });
    // Return the updated inspector content
    let markup = render_inspector().await;
    markup
}

async fn render_inspector() -> Html<String> {
    AsyncWorld.run(|world| -> Html<String> {
        let markup = html! {
            div class="inspector-container" {
                @if let Some(selected_entity) = world.resource::<SelectedEntity>().0 {
                    (render_component_list(selected_entity, &world))
                } @else {
                    p class="text-neutral-300 text-sm" { "Select an entity to inspect" }
                }
            }
        };

        Html(markup.into_string())
    })
}

fn render_component_list(entity: Entity, world: &World) -> Markup {
    let type_registry = world.resource::<AppTypeRegistry>().read();

    html! {
        div class="component-list" {
            @for component_info in world.inspect_entity(entity) {
                (render_component(component_info.clone(), &type_registry))
            }
        }
    }
}

fn render_component(component_info: ComponentInfo, type_registry: &TypeRegistry) -> Markup {
    let (_, name) = component_info.name().rsplit_once("::").unwrap();
    let type_info = component_info
        .type_id()
        .and_then(|type_id| type_registry.get_type_info(type_id));

    html! {
        div class="component-card card" {
            h3 class="text-sm font-medium" { (name) }

            @if let Some(type_info) = type_info {
                (render_type_info(type_info))
            } @else {
                p class="text-neutral-300 text-xs" { "Reflect not implemented" }
            }
        }
    }
}

fn render_type_info(type_info: &TypeInfo) -> Markup {
    match type_info {
        TypeInfo::Struct(info) => render_struct(info),
        TypeInfo::TupleStruct(info) => render_tuple_struct(info),
        TypeInfo::Enum(info) => render_enum(info),
        _ => html! { p { "Type not yet supported" } },
    }
}

fn render_struct(struct_info: &StructInfo) -> Markup {
    html! {
        div class="struct-fields" {
            @for field in struct_info.iter() {
                div class="field-row" {
                    label class="text-xs" { (field.name()) }
                    input type="text"
                          name=(field.name())
                          value=""
                          hx-put={"/component/" (field.name())}
                          hx-headers=(PreEscaped(r#"{"Content-Type": "application/json"}"#))
                          parse-types="true"
                          hx-ext="json-enc-custom"
                          hx-trigger="change" {}
                }
            }
        }
    }
}

fn render_tuple_struct(tuple_struct_info: &TupleStructInfo) -> Markup {
    html! {
        div class="tuple-struct-fields" {
            @for (idx, field) in tuple_struct_info.iter().enumerate() {
                div class="field-row" {
                    label class="text-xs" { (idx) }
                    input type="text"
                          name=(idx.to_string())
                          value=""
                          hx-put={"/component/" (idx)}
                          hx-headers=(PreEscaped(r#"{"Content-Type": "application/json"}"#))
                          parse-types="true"
                          hx-ext="json-enc-custom"
                          hx-trigger="change" {}
                }
            }
        }
    }
}

fn render_enum(enum_info: &EnumInfo) -> Markup {
    html! {
        div class="enum-variants" {
            select class="variant-select"
                   hx-put="/component/variant"
                   hx-headers=(PreEscaped(r#"{"Content-Type": "application/json"}"#))
                   parse-types="true"
                   hx-ext="json-enc-custom"
                   hx-trigger="change" {
                @for variant in enum_info.iter() {
                    option value=(variant.name()) { (variant.name()) }
                }
            }
        }
    }
}

// Vector3 input component example
fn render_vec3_input(entity: Entity, field_name: &str, value: Vec3) -> Markup {
    html! {
        div class="vector-input transform-grid" {
            div class="vector-label" { (field_name) }
            @for (component, val) in [("x", value.x), ("y", value.y), ("z", value.z)] {
                input type="number"
                      name=(component)
                      value=(val)
                      step="10"
                      hx-put={"/transform/" (serde_json::to_string(&entity).unwrap()) "/" (field_name)}
                      hx-headers=(PreEscaped(r#"{"Content-Type": "application/json"}"#))
                      parse-types="true"
                      hx-ext="json-enc-custom"
                      hx-trigger="change" {}
            }
        }
    }
}

async fn update_component(
    axum::extract::Path((entity, component)): axum::extract::Path<(Entity, String)>,
    Json(value): Json<ComponentValue>,
) -> Html<String> {
    // Update component logic here
    // Return updated component markup
    Html("Updated".to_string())
}

async fn delete_component(
    axum::extract::Path(entity): axum::extract::Path<Entity>,
) -> Html<String> {
    // Delete component logic here
    Html("".to_string())
}

// CSS styles for the inspector
const INSPECTOR_STYLES: &str = r#"
.inspector-container {
    padding: 1rem;
    background-color: rgb(82 82 91);
    height: 100%;
    overflow-y: auto;
}

.component-card {
    background-color: rgb(63 63 70);
    padding: 0.75rem;
    border-radius: 0.375rem;
    margin-bottom: 0.5rem;
}

.field-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
}

.vector-input {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.25rem;
}

input[type="number"],
input[type="text"],
select {
    background-color: rgb(39 39 42);
    color: white;
    border: 1px solid rgb(82 82 91);
    border-radius: 0.25rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.875rem;
    width: 100%;
}

.vector-label {
    grid-column: span 3;
    font-size: 0.75rem;
    color: rgb(212 212 216);
}
"#;
