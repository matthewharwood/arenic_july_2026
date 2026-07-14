use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};

use bevy::prelude::*;

use crate::{
    hero::{Active, Hero, Selected},
    tile,
};

const ARENA_ZOOM: f32 = 24.0;
const OVERWORLD_ZOOM: f32 = 72.0;
const CAMERA_ZOOMS: [f32; 2] = [ARENA_ZOOM, OVERWORLD_ZOOM];
const DEFAULT_CAMERA_ZOOM: f32 = CAMERA_ZOOMS[0];

const SHOULDER_BACK_DISTANCE: f32 = tile::SIZE * 3.0;
const SHOULDER_RIGHT_OFFSET: f32 = tile::SIZE * 0.9;
const SHOULDER_HEIGHT: f32 = tile::SIZE;
const SHOULDER_LOOK_AHEAD_DISTANCE: f32 = tile::SIZE * 4.0;

type ActiveSelectedHero = (With<Hero>, With<Selected>, With<Active>);

/// Adds the game's camera angles and active-camera behavior.
pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraView>()
            .add_systems(Startup, (spawn_top_down_camera, spawn_over_shoulder_camera))
            .add_systems(Update, (attach_over_shoulder_camera, toggle_camera_view));
    }
}

/// The camera composition currently presented to the player.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CameraView {
    Arena,
    #[default]
    OverShoulder,
}

impl CameraView {
    const fn toggled(self) -> Self {
        match self {
            Self::Arena => Self::OverShoulder,
            Self::OverShoulder => Self::Arena,
        }
    }
}

/// Marks the retained top-down arena/overworld camera.
#[derive(Component, Debug, Clone, Copy)]
struct TopDownCamera;

/// Marks the active camera that follows the selected hero's right shoulder.
#[derive(Component, Debug, Clone, Copy)]
struct OverShoulderCamera;

fn spawn_top_down_camera(mut commands: Commands, camera_view: Res<CameraView>) {
    commands.spawn((
        TopDownCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FRAC_PI_8,
            near: 0.05,
            far: 150.0,
            ..default()
        }),
        Camera {
            is_active: matches!(*camera_view, CameraView::Arena),
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.03)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, DEFAULT_CAMERA_ZOOM).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_over_shoulder_camera(mut commands: Commands, camera_view: Res<CameraView>) {
    commands.spawn((
        OverShoulderCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FRAC_PI_4,
            near: 0.05,
            far: 150.0,
            ..default()
        }),
        Camera {
            is_active: matches!(*camera_view, CameraView::OverShoulder),
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.03)),
            ..default()
        },
        over_shoulder_local_transform(),
    ));
}

/// Reparents the over-the-shoulder camera to the active selected hero.
fn attach_over_shoulder_camera(
    mut commands: Commands,
    active_selected_hero: Single<Entity, ActiveSelectedHero>,
    over_shoulder_camera: Single<(Entity, Option<&ChildOf>), With<OverShoulderCamera>>,
) {
    let hero = active_selected_hero.into_inner();
    let (camera, parent) = over_shoulder_camera.into_inner();

    if parent.is_none_or(|parent| parent.parent() != hero) {
        commands.entity(camera).insert(ChildOf(hero));
    }
}

fn toggle_camera_view(
    gamepads: Query<&Gamepad>,
    mut camera_view: ResMut<CameraView>,
    mut top_down_camera: Single<&mut Camera, (With<TopDownCamera>, Without<OverShoulderCamera>)>,
    mut over_shoulder_camera: Single<
        &mut Camera,
        (With<OverShoulderCamera>, Without<TopDownCamera>),
    >,
) {
    if !gamepads
        .iter()
        .any(|gamepad| gamepad.just_pressed(GamepadButton::LeftThumb))
    {
        return;
    }

    *camera_view = camera_view.toggled();
    top_down_camera.is_active = matches!(*camera_view, CameraView::Arena);
    over_shoulder_camera.is_active = matches!(*camera_view, CameraView::OverShoulder);
}

fn over_shoulder_local_transform() -> Transform {
    let camera_position = Vec3::new(
        SHOULDER_RIGHT_OFFSET,
        -SHOULDER_BACK_DISTANCE,
        SHOULDER_HEIGHT,
    );
    let look_at = Vec3::new(SHOULDER_RIGHT_OFFSET, SHOULDER_LOOK_AHEAD_DISTANCE, 0.0);

    Transform::from_translation(camera_position).looking_at(look_at, Vec3::Z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shoulder_camera_local_position_is_behind_above_and_to_the_heros_right() {
        let camera = over_shoulder_local_transform();

        assert!(camera.translation.x > 0.0);
        assert!(camera.translation.y < 0.0);
        assert!(camera.translation.z > 0.0);
    }

    #[test]
    fn shoulder_camera_local_transform_is_finite() {
        let camera = over_shoulder_local_transform();

        assert!(camera.translation.is_finite());
        assert!(camera.rotation.is_finite());
    }

    #[test]
    fn camera_view_toggle_alternates_between_both_views() {
        assert_eq!(CameraView::OverShoulder.toggled(), CameraView::Arena);
        assert_eq!(CameraView::Arena.toggled(), CameraView::OverShoulder);
    }

    #[test]
    fn shoulder_camera_reparents_to_the_active_selected_hero() {
        let mut app = App::new();
        app.add_systems(Update, attach_over_shoulder_camera);

        let first_hero = app.world_mut().spawn((Hero, Selected, Active)).id();
        let second_hero = app.world_mut().spawn(Hero).id();
        let local_transform = over_shoulder_local_transform();
        let camera = app
            .world_mut()
            .spawn((OverShoulderCamera, local_transform))
            .id();

        app.update();
        assert_eq!(camera_parent(&app, camera), first_hero);

        app.world_mut()
            .entity_mut(first_hero)
            .remove::<(Selected, Active)>();
        app.world_mut()
            .entity_mut(second_hero)
            .insert((Selected, Active));
        app.update();

        assert_eq!(camera_parent(&app, camera), second_hero);
        assert_eq!(app.world().get::<Transform>(camera), Some(&local_transform));
    }

    #[test]
    fn left_stick_click_activates_only_the_arena_camera() {
        let mut app = App::new();
        app.init_resource::<CameraView>()
            .add_systems(Update, toggle_camera_view);

        let top_down_camera = app
            .world_mut()
            .spawn((
                TopDownCamera,
                Camera {
                    is_active: false,
                    ..default()
                },
            ))
            .id();
        let over_shoulder_camera = app
            .world_mut()
            .spawn((OverShoulderCamera, Camera::default()))
            .id();
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();

        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::LeftThumb);
        app.update();

        assert_eq!(*app.world().resource::<CameraView>(), CameraView::Arena);
        assert!(
            app.world()
                .get::<Camera>(top_down_camera)
                .is_some_and(|camera| camera.is_active)
        );
        assert!(
            app.world()
                .get::<Camera>(over_shoulder_camera)
                .is_some_and(|camera| !camera.is_active)
        );
    }

    fn camera_parent(app: &App, camera: Entity) -> Entity {
        app.world()
            .get::<ChildOf>(camera)
            .expect("invariant: the shoulder camera is attached to a hero")
            .parent()
    }
}
