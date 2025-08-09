# `bevy_world_space_ui`

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Katsutoshii/bevy_world_space_ui#license)
[![Crates.io](https://img.shields.io/crates/v/bevy_world_space_ui.svg)](https://crates.io/crates/bevy_world_space_ui)
[![Docs](https://docs.rs/bevy_world_space_ui/badge.svg)](https://docs.rs/bevy_world_space_ui/latest/bevy_world_space_ui/)

Make building world space UIs easier in Bevy game engine.

## Usage

```rs
use bevy_world_space_ui::{WorldSpaceUiPlugin, WorldSpaceUiRoot, WorldSpaceUiSurface};

// Generate your own pointer ID.
const WORLD_SPACE_UI_POINTER: PointerId = PointerId::Custom(Uuid::from_u128(123));

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldSpaceUiPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create the texture to be rendered to.
    let resolution = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let image_handle = images.add(WorldSpaceUiRoot::get_ui_texture(resolution));

    // Spawn your UI with `WorldSpaceUiRoot`.
    let root = commands
        .spawn((
            WorldSpaceUiRoot {
                texture: image_handle.clone(),
            },
            // Spawn your UI tree...
        ))
        .id();

    // Spawn a quad to render the UI on.
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        WorldSpaceUiSurface {
            root,
            texture: image_handle.clone(),
            pointer_id: WORLD_SPACE_UI_POINTER,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.5).with_rotation(Quat::from_axis_angle(Vec3::X, PI / 8.)),
    ));

    // Setup cameras and lights as you normally would.
    // ...
}

```

See `examples` for a working demo.

## Bevy support table

| bevy | bevy_world_space_ui  |
| ---- | -------------------- |
| 0.16 | 0.1.3                |
