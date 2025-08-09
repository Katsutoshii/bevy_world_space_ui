//! Utilities for creating world space UIs in Bevy.
use bevy::{
    app::{App, First, Plugin},
    asset::{Assets, Handle, RenderAssetUsages},
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::{Component, HookContext},
        entity::Entity,
        error::Result,
        event::{EventReader, EventWriter},
        name::Name,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
        world::DeferredWorld,
    },
    image::Image,
    input::{ButtonState, mouse::MouseButton},
    math::{UVec2, Vec2, Vec3Swizzles},
    pbr::{MeshMaterial3d, StandardMaterial},
    picking::{
        PickSet,
        backend::ray::RayMap,
        mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility, RayMeshHit},
        pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput},
    },
    reflect::Reflect,
    render::{
        camera::{Camera, ClearColorConfig, NormalizedRenderTarget, RenderTarget},
        mesh::{Indices, Mesh, Mesh3d, VertexAttributeValues},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
    ui::UiTargetCamera,
    utils::default,
    window::{PrimaryWindow, WindowEvent},
};

/// Plugin supporting world space UI.
#[derive(Default)]
pub struct WorldSpaceUiPlugin;
impl Plugin for WorldSpaceUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WorldSpaceUiRoot>()
            .register_type::<WorldSpaceUiSurface>()
            .add_systems(
                First,
                (drive_diegetic_pointer, send_pointer_input)
                    .chain()
                    .in_set(PickSet::Input),
            );
    }
}

/// Marks the root node of a UI tree that is rendered to a texture for
/// display in world space.
/// This automatically spawns a render camera and adds a `UiTargetCamera` component.
#[derive(Component, Debug, Clone, Reflect)]
#[component(on_add = WorldSpaceUiRoot::on_add)]
pub struct WorldSpaceUiRoot {
    pub texture: Handle<Image>,
}
impl WorldSpaceUiRoot {
    /// Constructs a UI texture for rendering world space UI.
    pub fn get_ui_texture(resolution: Extent3d) -> Image {
        // This is the texture that will be rendered to.
        let mut image = Image::new_fill(
            resolution,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );
        // You need to set these texture usage flags in order to use the image as a render target
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        image
    }

    /// Automatically spawns a UI target camera and render target for the UI root.
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let root = world.entity(context.entity).components::<&Self>().clone();
        let texture_camera = world
            .commands()
            .spawn((
                Name::new("UiTargetCamera"),
                Camera2d,
                Camera {
                    target: RenderTarget::Image(root.texture.clone().into()),
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
            ))
            .id();
        world
            .commands()
            .entity(context.entity)
            .insert(UiTargetCamera(texture_camera));
    }
}

/// Stores render target information for a `WorldSpaceUiSurface`.
#[derive(Component, Debug, Clone)]
pub struct WorldSpaceUiRenderTarget {
    pub target: NormalizedRenderTarget,
    pub size: UVec2,
}

/// Persists the previous cursor position on a `WorldSpaceUiSurface`.
#[derive(Component, Debug, Clone, Default)]
struct PreviousCursorPosition(pub Vec2);

/// Marks a mesh as a surface where UI will be rendered and interacted with.
#[derive(Component, Debug, Clone, Reflect)]
#[require(Mesh3d, PreviousCursorPosition)]
#[component(on_add = WorldSpaceUiSurface::on_add)]
pub struct WorldSpaceUiSurface {
    pub root: Entity,
    pub texture: Handle<Image>,
    pub pointer_id: PointerId,
    pub default_material: Option<StandardMaterial>,
}
impl Default for WorldSpaceUiSurface {
    fn default() -> Self {
        Self {
            root: Entity::PLACEHOLDER,
            texture: Handle::default(),
            pointer_id: PointerId::default(),
            default_material: None,
        }
    }
}
impl WorldSpaceUiSurface {
    /// On component add, attach a MeshMaterial3d using the image
    /// and spawn a UI camera and custom pointer.
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let surface = world.entity(context.entity).components::<&Self>().clone();

        // This material has the texture that has been rendered.
        let material_handle =
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color_texture: Some(surface.texture.clone()),
                    ..surface.default_material.unwrap_or_default()
                });

        let primary_window = world
            .try_query_filtered::<Entity, With<PrimaryWindow>>()
            .unwrap()
            .single(&world)
            .ok();
        let ui_camera_entity = world.entity(surface.root).components::<&UiTargetCamera>().0;
        let ui_camera = world.entity(ui_camera_entity).components::<&Camera>();
        let target = ui_camera.target.normalize(primary_window).unwrap();
        let size = world
            .resource::<Assets<Image>>()
            .get(&surface.texture)
            .unwrap()
            .size();

        world
            .commands()
            .entity(context.entity)
            .insert(MeshMaterial3d(material_handle))
            .insert(WorldSpaceUiRenderTarget { target, size });

        // Spawn a virtual pointer so we can send events to the rendered UI.
        world.commands().spawn(surface.pointer_id);
    }

    /// Computes the UV coordinates of a ray-mesh hit.
    pub fn get_ray_mesh_hit_uv(hit: &RayMeshHit, mesh: &Mesh) -> Option<Vec2> {
        let uvs = mesh.attribute(Mesh::ATTRIBUTE_UV_0);
        let Some(VertexAttributeValues::Float32x2(uvs)) = uvs else {
            return None;
        };

        let uvs: [Vec2; 3] = if let Some(indices) = mesh.indices() {
            let i = hit.triangle_index.unwrap() * 3;
            match indices {
                Indices::U16(indices) => [
                    Vec2::from(uvs[indices[i] as usize]),
                    Vec2::from(uvs[indices[i + 1] as usize]),
                    Vec2::from(uvs[indices[i + 2] as usize]),
                ],
                Indices::U32(indices) => [
                    Vec2::from(uvs[indices[i] as usize]),
                    Vec2::from(uvs[indices[i + 1] as usize]),
                    Vec2::from(uvs[indices[i + 2] as usize]),
                ],
            }
        } else {
            let i = hit.triangle_index.unwrap() * 3;
            [
                Vec2::from(uvs[i]),
                Vec2::from(uvs[i + 1]),
                Vec2::from(uvs[i + 2]),
            ]
        };

        let bc = hit.barycentric_coords.zxy();
        let uv = bc.x * uvs[0] + bc.y * uvs[1] + bc.z * uvs[2];
        Some(uv)
    }
}

/// Because bevy has no way to know how to map a mouse input to the UI texture, we need to write a
/// system that tells it there is a pointer on the UI texture. We cast a ray into the scene and find
/// the UV (2D texture) coordinates of the raycast hit. This UV coordinate is effectively the same
/// as a pointer coordinate on a 2D UI rect.
fn drive_diegetic_pointer(
    mut raycast: MeshRayCast,
    rays: Res<RayMap>,
    surfaces_check: Query<Entity, With<WorldSpaceUiSurface>>,
    mut surfaces: Query<(
        &WorldSpaceUiSurface,
        &WorldSpaceUiRenderTarget,
        &Mesh3d,
        &mut PreviousCursorPosition,
    )>,
    meshes: Res<Assets<Mesh>>,
    mut pointer_input: EventWriter<PointerInput>,
) -> Result {
    // Find raycast hits and update the virtual pointer.
    let raycast_settings = MeshRayCastSettings {
        visibility: RayCastVisibility::VisibleInView,
        filter: &|entity| surfaces_check.contains(entity),
        early_exit_test: &|_| false,
    };
    let mut hit_pointer_ids = Vec::new();

    for (_id, ray) in rays.iter() {
        for (cube, hit) in raycast.cast_ray(*ray, &raycast_settings) {
            let (surface, render_target, mesh_handle, mut cursor_last) = surfaces.get_mut(*cube)?;
            let mesh = meshes.get(mesh_handle).unwrap();
            let Some(uv) = WorldSpaceUiSurface::get_ray_mesh_hit_uv(hit, mesh) else {
                continue;
            };
            hit_pointer_ids.push(surface.pointer_id);
            let position = render_target.size.as_vec2() * uv;
            if position != cursor_last.0 {
                pointer_input.write(PointerInput::new(
                    surface.pointer_id,
                    Location {
                        target: render_target.target.clone(),
                        position,
                    },
                    PointerAction::Move {
                        delta: position - cursor_last.0,
                    },
                ));
                cursor_last.0 = position;
            }
        }
    }

    Ok(())
}

/// Send pointer pressed and released events to the world space UI.
fn send_pointer_input(
    surfaces: Query<(
        &WorldSpaceUiSurface,
        &WorldSpaceUiRenderTarget,
        &PreviousCursorPosition,
    )>,
    mut window_events: EventReader<WindowEvent>,
    mut pointer_input: EventWriter<PointerInput>,
) {
    // Pipe pointer button presses to the virtual pointer on the UI texture.
    for window_event in window_events.read() {
        if let WindowEvent::MouseButtonInput(input) = window_event {
            let button = match input.button {
                MouseButton::Left => PointerButton::Primary,
                MouseButton::Right => PointerButton::Secondary,
                MouseButton::Middle => PointerButton::Middle,
                _ => continue,
            };
            let action = match input.state {
                ButtonState::Pressed => PointerAction::Press(button),
                ButtonState::Released => PointerAction::Release(button),
            };
            for (surface, render_target, cursor_last) in surfaces.iter() {
                pointer_input.write(PointerInput::new(
                    surface.pointer_id,
                    Location {
                        target: render_target.target.clone(),
                        position: cursor_last.0,
                    },
                    action,
                ));
            }
        }
    }
}
