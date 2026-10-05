use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{
    response::Html,
    routing::{delete, get, put},
    Json,
};
use bevy::ecs::component::ComponentInfo;
use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy::reflect::{
    enums::EnumInfo, structs::StructInfo, tuple_struct::TupleStructInfo, ReflectFromPtr, TypeInfo,
    TypeRegistry,
};
use bevy_defer::AsyncWorld;
use bevy_webgate::prelude::*;
use maud::{html, Markup};

pub struct EditorCorePlugin;

impl Plugin for EditorCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntity>()
            .register_type::<SelectedEntity>()
            .add_systems(PostUpdate, reset_selected_entity_if_entity_despawned);
    }
}

/// The currently selected entity in the scene.
#[derive(Default, Reflect, Resource)]
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
        .add_plugins((DefaultPlugins, EditorCorePlugin, WebInspectorPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((Name::new("owo"), Transform::default()));
}

pub struct WebInspectorPlugin;

impl Plugin for WebInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.route("/", get(render_layout))
            .route("/inspector", get(render_inspector))
            .route(
                "/component/{entity}/{component}/{field-name}",
                put(update_component_field),
            )
            .route("/component/{entity}", delete(delete_component))
            .route("/entities", get(render_entity_list))
            .route("/entities/select/{entity}", post(select_entity));
    }
}

#[derive(serde::Deserialize)]
struct FieldUpdate {
    value: serde_json::Value,
}

async fn update_component_field(
    Path((entity_index, component_name, field_name)): Path<(u32, String, String)>,
    Json(update): Json<FieldUpdate>,
) -> ([(&'static str, &'static str); 1], StatusCode) {
    let status = AsyncWorld.run(|world| {
        let entity = Entity::from_raw_u32(entity_index).unwrap_or(Entity::PLACEHOLDER);

        match apply_field_update(world, entity, &component_name, &field_name, update.value) {
            Some(()) => StatusCode::NO_CONTENT,
            None => StatusCode::UNPROCESSABLE_ENTITY,
        }
    });
    ([("HX-Trigger", "entity-list-changed")], status)
}

fn apply_field_update(
    world: &mut World,
    entity: Entity,
    component_name: &str,
    field_name: &str,
    value: serde_json::Value,
) -> Option<()> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    if component_name == "Name" && field_name == "value" {
        let name = serde_json::from_value::<String>(value).ok()?;
        world.get_mut::<Name>(entity)?.set(name);
        return Some(());
    }

    let (type_id, id) = world.components().iter_registered().find_map(|component| {
        let type_id = component.type_id()?;
        let info = type_registry.get_type_info(type_id)?;
        let short_name = info.type_path().split("::").last().unwrap_or("");
        (short_name == component_name).then_some((type_id, component.id()))
    })?;

    let mut entity_mut = world.get_entity_mut(entity).ok()?;
    let mut component_ref = entity_mut.get_mut_by_id(id).ok()?;
    let reflect_from_ptr = type_registry.get(type_id)?.data::<ReflectFromPtr>()?;
    // SAFE: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
    let reflected = unsafe { reflect_from_ptr.as_reflect_mut(component_ref.as_mut()) };
    drop(type_registry);
    let struct_info = reflected.reflect_mut().as_struct().ok()?;
    let field = struct_info.field_mut(field_name)?;
    let field_type_name = field.try_as_reflect()?.reflect_type_ident()?;

    match field_type_name {
        "Vec3" => {
            let [x, y, z] = serde_json::from_value::<[f32; 3]>(value).ok()?;
            field.try_apply(&Vec3::new(x, y, z)).ok()?;
        }
        "f32" => {
            let float_value = serde_json::from_value::<f32>(value).ok()?;
            field.try_apply(&float_value).ok()?;
        }
        "String" => {
            let string_value = serde_json::from_value::<String>(value).ok()?;
            field.try_apply(&string_value).ok()?;
        }
        "bool" => {
            let bool_value = serde_json::from_value::<bool>(value).ok()?;
            field.try_apply(&bool_value).ok()?;
        }
        "Color" => {
            let [r, g, b, a] = serde_json::from_value::<[f32; 4]>(value).ok()?;
            field.try_apply(&Color::srgba(r, g, b, a)).ok()?;
        }
        "Quat" => {
            let [x, y, z, w] = serde_json::from_value::<[f32; 4]>(value).ok()?;
            field.try_apply(&Quat::from_xyzw(x, y, z, w)).ok()?;
        }
        // Add more type handlers as needed
        _ => return None,
    }

    Some(())
}

// ... [Previous plugin and struct definitions remain the same until render_layout]

async fn render_layout() -> Html<String> {
    Html(
        html! {
            html {
                head {
                    title { "Bevy Web Inspector" }
                    script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.11/dist/htmx.min.js"
                        integrity="sha384-2OatzQy1H+Zd/IIrjr1TcuDGqLXeHhbooAyJY1KdQMKnr4LZ22k31GBLdYKHmVjg"
                        crossorigin="anonymous" {}
                    script src="https://cdn.jsdelivr.net/gh/Emtyloc/json-enc-custom@main/json-enc-custom.js" {}
                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/bootstrap/5.3.2/css/bootstrap.min.css" {}
                    script src="https://cdnjs.cloudflare.com/ajax/libs/bootstrap/5.3.2/js/bootstrap.bundle.min.js" {}
                    style { (INSPECTOR_STYLES) }
                }
                body class="bg-dark" {
                    div class="container-fluid vh-100 p-0" {
                        div class="row h-100 g-0" {
                            // Entity list panel
                            div class="col-3 border-end border-secondary"
                                hx-get="/entities"
                                hx-trigger="load, entity-list-changed from:body"
                                hx-swap="innerHTML" {}
                            // Inspector panel
                            div id="inspector"
                                class="col-9"
                                hx-get="/inspector"
                                hx-trigger="load"
                                hx-swap="innerHTML" {}
                        }
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
            div class="entity-list p-3 bg-dark" {
                h2 class="h4 text-light mb-4" { "Entities" }

                div class="entity-cards" {
                    @for (entity, name) in get_named_entities(world) {
                        form class="card bg-secondary mb-3"
                             hx-post=(format!("/entities/select/{}", entity.index()))
                             hx-target="#inspector"
                             hx-swap="innerHTML" {

                            div class="card-body" {
                                // Entity info section
                                div class="d-flex justify-content-between align-items-center mb-2" {
                                    div {
                                        span class="badge bg-dark text-light" {
                                            "#" (entity.index())
                                        }
                                        @if let Some(name) = &name {
                                            span class="ms-2 text-light" {
                                                (name)
                                            }
                                        }
                                    }

                                    span class="badge bg-info" {
                                        (get_component_count(world, entity)) " components"
                                    }
                                }

                                button type="submit"
                                        class=(format!("btn btn-sm w-100 {}",
                                            if Some(entity) == selected {
                                                "btn-success"
                                            } else {
                                                "btn-outline-light"
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
            }
        };
        Html(markup.into_string())
    })
}

fn render_component(
    component_info: ComponentInfo,
    type_registry: &TypeRegistry,
    entity: Entity,
    world: &World, // Add world parameter
    _component_name: &str,
) -> Markup {
    let binding = component_info.name();
    let type_info = component_info
        .type_id()
        .and_then(|type_id| type_registry.get_type_info(type_id));
    let type_path = type_info.map_or(&*binding, |info| info.type_path());
    let name = type_path
        .rsplit_once("::")
        .map_or(type_path, |(_, name)| name);

    // Get the actual component data
    let component_data = if let Some(type_id) = component_info.type_id() {
        match world
            .entity(entity)
            .get_by_id(component_info.id())
            .map(|component| {
                let reflect_data = type_registry.get(type_id)?;
                let reflect_from_ptr = reflect_data.data::<ReflectFromPtr>()?;
                Some(unsafe { reflect_from_ptr.as_reflect(component) })
            }) {
            Ok(Some(awa)) => Some(awa),
            _ => None,
        }
    } else {
        return html! {};
    };

    html! {
        div class="card bg-secondary mb-3" {
            div class="card-header" {
                h4 class="card-title h6 mb-0 text-light" { (name) }
            }
            div class="card-body" {
                @if let (Some(type_info), Some(component_data)) = (type_info, component_data) {
                    (render_type_info(type_info, entity, name, component_data))
                } @else {
                    p class="text-light small mb-0" { "Reflect not implemented" }
                }
            }
        }
    }
}

fn render_component_list(entity: Entity, world: &World) -> Markup {
    let type_registry = world.resource::<AppTypeRegistry>().read();

    let components = world.inspect_entity(entity).into_iter().flatten();

    html! {
        div class="component-list p-3" {
            h3 class="h5 text-light mb-3" { "Entity Components" }
            @for component_info in components {
                (render_component(
                    component_info.clone(),
                    &type_registry,
                    entity,
                    world,  // Pass world to render_component
                    &component_info.name()
                ))
            }
        }
    }
}

fn render_struct(
    struct_info: &StructInfo,
    entity: Entity,
    component_name: &str,
    component_data: &dyn Reflect,
) -> Markup {
    let Ok(struct_data) = component_data.reflect_ref().as_struct() else {
        return html! {};
    };

    html! {
        div class="struct-fields card bg-secondary" {
            div class="card-body" {
                @for field in struct_info.iter() {
                    form class="mb-3"
                        hx-put={"/component/" (entity.index()) "/" (component_name) "/" (field.name())}
                        parse-types="true"
                        hx-ext="json-enc-custom"
                        hx-trigger="change"
                        hx-swap="none" {
                        label class="form-label text-light small" { (field.name()) }
                        @let field_data = struct_data.field(field.name());
                        @let coordinates = field_data.and_then(|value| {
                            value.try_downcast_ref::<Vec3>().map(|vector| vec![("x", vector.x), ("y", vector.y), ("z", vector.z)]).or_else(|| {
                                value.try_downcast_ref::<Quat>().map(|rotation| vec![("x", rotation.x), ("y", rotation.y), ("z", rotation.z), ("w", rotation.w)])
                            })
                        });
                        @if let Some(coordinates) = coordinates {
                            div class="row g-2" {
                                @for (axis, value) in coordinates {
                                    div class="col" {
                                        input type="number"
                                            class="form-control form-control-sm bg-dark text-light border-secondary"
                                            name="value"
                                            aria-label=(axis)
                                            value=(value)
                                            step="0.1"
                                            required {}
                                    }
                                }
                            }
                        } @else {
                            @let editable = field_data.is_some_and(|value| value.try_downcast_ref::<f32>().is_some() || value.try_downcast_ref::<bool>().is_some() || value.try_downcast_ref::<String>().is_some());
                            @let field_value = field_data.map_or_else(String::new, |value| {
                                value.try_downcast_ref::<String>().map_or_else(|| format!("{value:?}"), Clone::clone)
                            });
                            input type=(if field_data.is_some_and(|value| value.try_downcast_ref::<f32>().is_some()) { "number" } else if field_data.is_some_and(|value| value.try_downcast_ref::<bool>().is_some()) { "checkbox" } else { "text" })
                                class="form-control form-control-sm bg-dark text-light border-secondary"
                                name="value"
                                value=(field_value)
                                checked[field_data.and_then(|value| value.try_downcast_ref::<bool>()).copied().unwrap_or(false)]
                                readonly[!editable]
                                step="any" {}
                        }
                    }
                }
            }
        }
    }
}

fn render_enum(enum_info: &EnumInfo, component_data: &dyn Reflect) -> Markup {
    let current_variant = component_data
        .reflect_ref()
        .as_enum()
        .ok()
        .map(bevy::reflect::enums::Enum::variant_name);
    html! {
        div class="enum-variants" {
            select class="form-select form-select-sm bg-dark text-light border-secondary"
                   disabled {
                @for variant in enum_info.iter() {
                    option value=(variant.name()) selected[current_variant == Some(variant.name())] { (variant.name()) }
                }
            }
        }
    }
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
        if world.inspect_entity(entity).is_ok_and(|mut components| {
            components.any(|info| {
                let type_register = type_registry.clone();
                let type_register = type_register.read();
                info.type_id()
                    .is_some_and(|type_id| type_register.get_type_info(type_id).is_some())
            })
        }) {
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
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.clone();

    // Only count reflected components
    world.inspect_entity(entity).map_or(0, |components| {
        components
            .filter(|info| {
                let type_registry = type_registry.read();
                info.type_id()
                    .is_some_and(|type_id| type_registry.get_type_info(type_id).is_some())
            })
            .count()
    })
}

async fn select_entity(
    axum::extract::Path(entity_index): axum::extract::Path<u32>,
) -> ([(&'static str, &'static str); 1], Html<String>) {
    AsyncWorld.run(|world| {
        // Create entity from index and update selected entity
        let entity = Entity::from_raw_u32(entity_index).unwrap_or(Entity::PLACEHOLDER);
        if world.get_entity(entity).is_ok() {
            world.resource_mut::<SelectedEntity>().0 = Some(entity);
        }
    });
    // Return the updated inspector content
    let markup = render_inspector().await;
    ([("HX-Trigger", "entity-list-changed")], markup)
}

async fn render_inspector() -> Html<String> {
    AsyncWorld.run(|world| -> Html<String> {
        let markup = html! {
            div class="inspector-container" {
                @if let Some(selected_entity) = world.resource::<SelectedEntity>().0 {
                    (render_component_list(selected_entity, world))
                } @else {
                    p class="text-neutral-300 text-sm" { "Select an entity to inspect" }
                }
            }
        };

        Html(markup.into_string())
    })
}

fn render_type_info(
    type_info: &TypeInfo,
    entity: Entity,
    component_name: &str,
    component_data: &dyn Reflect,
) -> Markup {
    match type_info {
        TypeInfo::Struct(info) => render_struct(info, entity, component_name, component_data),
        TypeInfo::TupleStruct(info) => {
            render_tuple_struct(info, entity, component_name, component_data)
        }
        TypeInfo::Enum(info) => render_enum(info, component_data),
        TypeInfo::Tuple(_)
        | TypeInfo::List(_)
        | TypeInfo::Array(_)
        | TypeInfo::Map(_)
        | TypeInfo::Set(_)
        | TypeInfo::Opaque(_) => html! { p { "Type not yet supported" } },
    }
}

fn render_tuple_struct(
    tuple_struct_info: &TupleStructInfo,
    entity: Entity,
    component_name: &str,
    component_data: &dyn Reflect,
) -> Markup {
    if let Some(name) = component_data.downcast_ref::<Name>() {
        return html! {
            form hx-put={"/component/" (entity.index()) "/" (component_name) "/value"}
                hx-ext="json-enc-custom"
                hx-trigger="change"
                hx-swap="none" {
                input type="text"
                    class="form-control form-control-sm bg-dark text-light border-secondary"
                    name="value"
                    aria-label="Name"
                    value=(name.as_str()) {}
            }
        };
    }
    let tuple_data = component_data.reflect_ref().as_tuple_struct().ok();
    html! {
        div class="tuple-struct-fields" {
            @for (idx, _field) in tuple_struct_info.iter().enumerate() {
                div class="field-row" {
                    label class="text-xs" { (idx) }
                    input type="text"
                          name=(idx.to_string())
                          value=(tuple_data.and_then(|value| value.field(idx)).map_or_else(String::new, |value| format!("{value:?}")))
                          readonly {}
                }
            }
        }
    }
}

async fn delete_component(axum::extract::Path(_entity): axum::extract::Path<Entity>) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflected_component_names_are_used_in_update_urls() {
        let mut app = App::new();
        app.register_type::<Transform>();
        let entity = app.world_mut().spawn(Transform::default()).id();

        let markup = render_component_list(entity, app.world()).into_string();

        assert!(markup.contains(">Transform</h4>"));
        assert!(markup.contains(&format!(
            "/component/{}/Transform/translation",
            entity.index()
        )));
        assert!(!markup.contains("Enable the debug feature"));
    }

    #[test]
    fn vector_update_is_reflected_when_rendered_again() {
        let mut app = App::new();
        app.register_type::<Transform>();
        let entity = app.world_mut().spawn(Transform::default()).id();

        assert_eq!(
            apply_field_update(
                app.world_mut(),
                entity,
                "Transform",
                "translation",
                serde_json::json!([125.1, 2.0, 3.0])
            ),
            Some(())
        );
        let markup = render_component_list(entity, app.world()).into_string();
        assert!(markup.contains("name=\"value\" aria-label=\"x\" value=\"125.1\""));
        assert!(markup.contains("hx-trigger=\"change\" hx-swap=\"none\""));
        assert_eq!(
            app.world()
                .get::<Transform>(entity)
                .map(|transform| transform.translation),
            Some(Vec3::new(125.1, 2.0, 3.0))
        );
    }

    #[test]
    fn invalid_vector_update_is_rejected() {
        let mut app = App::new();
        app.register_type::<Transform>();
        let entity = app.world_mut().spawn(Transform::default()).id();

        assert_eq!(
            apply_field_update(
                app.world_mut(),
                entity,
                "Transform",
                "translation",
                serde_json::json!("invalid")
            ),
            None
        );
        assert_eq!(
            app.world()
                .get::<Transform>(entity)
                .map(|transform| transform.translation),
            Some(Vec3::ZERO)
        );
    }

    #[test]
    fn name_update_refreshes_rendered_value() {
        let mut app = App::new();
        app.register_type::<Name>();
        let entity = app.world_mut().spawn(Name::new("before")).id();
        assert_eq!(
            apply_field_update(
                app.world_mut(),
                entity,
                "Name",
                "value",
                serde_json::json!("after")
            ),
            Some(())
        );
        assert_eq!(
            app.world().get::<Name>(entity).map(Name::as_str),
            Some("after")
        );
        let markup = render_component_list(entity, app.world()).into_string();
        assert!(markup.contains(&format!("/component/{}/Name/value", entity.index())));
        assert!(markup.contains("value=\"after\""));
    }

    #[test]
    fn rotation_update_renders_four_numeric_coordinates() {
        let mut app = App::new();
        app.register_type::<Transform>();
        let entity = app.world_mut().spawn(Transform::default()).id();
        assert_eq!(
            apply_field_update(
                app.world_mut(),
                entity,
                "Transform",
                "rotation",
                serde_json::json!([0.0, 0.0, 0.0, 1.0])
            ),
            Some(())
        );
        let markup = render_component_list(entity, app.world()).into_string();
        assert!(markup.contains("aria-label=\"w\" value=\"1\""));
    }
}
