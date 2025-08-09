//! Shows how to render UI to a texture. Useful for displaying UI in 3D space.

use std::f32::consts::PI;

use bevy::{
    asset::uuid::Uuid,
    color::palettes::css::{BLUE, GRAY, GREEN, RED},
    ecs::{component::HookContext, world::DeferredWorld},
    picking::pointer::PointerId,
    prelude::*,
    render::render_resource::Extent3d,
};
use bevy_world_space_ui::{WorldSpaceUiPlugin, WorldSpaceUiRoot, WorldSpaceUiSurface};

const WORLD_SPACE_UI_POINTER1: PointerId =
    PointerId::Custom(Uuid::from_u128(235172396560254989313697768709775153593));
const WORLD_SPACE_UI_POINTER2: PointerId =
    PointerId::Custom(Uuid::from_u128(235172396560254989313697768709775153594));

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldSpaceUiPlugin))
        .add_systems(Startup, setup)
        .run();
}

/// Hoverable button UI component.
#[derive(Component)]
#[require(
    Node {
        position_type: PositionType::Absolute,
        width: Val::Auto,
        height: Val::Auto,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(20.)),
        ..default()
    },
    BorderRadius::all(Val::Px(10.)),
    BackgroundColor(BLUE.into()),
)]
#[component(on_add = HoverableButton::on_add)]
pub struct HoverableButton {
    pub on_click_message: String,
}
impl HoverableButton {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(Self::on_over)
            .observe(Self::on_out)
            .observe(Self::on_click)
            .observe(Self::on_release)
            .with_child((
                Text::new("Click me!"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor::WHITE,
            ));
    }
    fn on_over(pointer: Trigger<Pointer<Over>>, mut colors: Query<&mut BackgroundColor>) {
        colors.get_mut(pointer.target()).unwrap().0 = RED.into();
    }
    fn on_release(pointer: Trigger<Pointer<Released>>, mut colors: Query<&mut BackgroundColor>) {
        colors.get_mut(pointer.target()).unwrap().0 = RED.into();
    }
    fn on_out(pointer: Trigger<Pointer<Out>>, mut colors: Query<&mut BackgroundColor>) {
        colors.get_mut(pointer.target()).unwrap().0 = BLUE.into();
    }
    fn on_click(
        pointer: Trigger<Pointer<Pressed>>,
        mut query: Query<(&Self, &mut BackgroundColor)>,
    ) {
        let (button, mut color) = query.get_mut(pointer.target()).unwrap();
        *color = GREEN.into();
        info!("{}", button.on_click_message);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create two UI textures, one for each quad.
    let resolution = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let image_handle1 = images.add(WorldSpaceUiRoot::get_ui_texture(resolution));
    let image_handle2 = images.add(WorldSpaceUiRoot::get_ui_texture(resolution));

    // Spawn two UIs.
    let root1 = commands
        .spawn((
            WorldSpaceUiRoot {
                texture: image_handle1.clone(),
            },
            Node {
                // Cover the whole image
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(GRAY.into()),
        ))
        .with_child(HoverableButton {
            on_click_message: "Button clicked 1".to_string(),
        })
        .id();
    let root2 = commands
        .spawn((
            WorldSpaceUiRoot {
                texture: image_handle2.clone(),
            },
            Node {
                // Cover the whole image
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(GRAY.into()),
        ))
        .with_child(HoverableButton {
            on_click_message: "Button clicked 2".to_string(),
        })
        .id();

    // Cube with material containing the rendered UI texture.
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        WorldSpaceUiSurface {
            root: root1,
            texture: image_handle1.clone(),
            pointer_id: WORLD_SPACE_UI_POINTER1,
            ..default()
        },
        Transform::from_xyz(-1.0, 0.0, 1.5).with_rotation(Quat::from_axis_angle(Vec3::X, PI / 8.)),
    ));

    // Cube with material containing the rendered UI texture.
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        WorldSpaceUiSurface {
            root: root2,
            texture: image_handle2.clone(),
            pointer_id: WORLD_SPACE_UI_POINTER2,
            ..default()
        },
        Transform::from_xyz(1.0, 0.0, 1.5).with_rotation(Quat::from_axis_angle(Vec3::X, PI / 8.)),
    ));

    // The main pass camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light.
    commands.spawn(DirectionalLight::default());
}
