use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};

use bevy::prelude::*;

use crate::{
    hero::{Active, Hero, Selected},
    tile,
};

const TOP_DOWN_CAMERA_DISTANCE: f32 = 24.0;

const TOP_DOWN_FIELD_OF_VIEW_RADIANS: f32 = FRAC_PI_8;
const OVER_SHOULDER_FIELD_OF_VIEW_RADIANS: f32 = FRAC_PI_4;
const CAMERA_NEAR_PLANE: f32 = 0.05;
const CAMERA_FAR_PLANE: f32 = 150.0;
const CAMERA_CLEAR_COLOR: Color = crate::theme::CANVAS;

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
            .add_systems(Startup, spawn_camera_scene)
            .add_systems(Update, (attach_over_shoulder_camera, toggle_camera_view));
    }
}

/// The camera composition currently presented to the player.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CameraView {
    #[default]
    Arena,
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
#[derive(Component, Debug, Default, Clone, Copy)]
struct TopDownCamera;

/// Marks the active camera that follows the selected hero's right shoulder.
#[derive(Component, Debug, Default, Clone, Copy)]
struct OverShoulderCamera;

fn spawn_camera_scene(mut commands: Commands, camera_view: Res<CameraView>) {
    commands.spawn_scene_list(camera_scene(*camera_view));
}

/// Describes both retained cameras for the requested initial view.
fn camera_scene(camera_view: CameraView) -> impl SceneList {
    let top_down_is_active = matches!(camera_view, CameraView::Arena);
    let over_shoulder_is_active = matches!(camera_view, CameraView::OverShoulder);

    bsn_list![
        top_down_camera_scene(top_down_is_active),
        over_shoulder_camera_scene(over_shoulder_is_active),
    ]
}

fn top_down_camera_scene(is_active: bool) -> impl Scene {
    bsn! {
        TopDownCamera
        camera_3d_scene(is_active, TOP_DOWN_FIELD_OF_VIEW_RADIANS)
        template_value(top_down_transform())
    }
}

fn over_shoulder_camera_scene(is_active: bool) -> impl Scene {
    bsn! {
        OverShoulderCamera
        camera_3d_scene(is_active, OVER_SHOULDER_FIELD_OF_VIEW_RADIANS)
        template_value(over_shoulder_local_transform())
    }
}

fn camera_3d_scene(is_active: bool, field_of_view_radians: f32) -> impl Scene {
    let projection = Projection::Perspective(PerspectiveProjection {
        fov: field_of_view_radians,
        near: CAMERA_NEAR_PLANE,
        far: CAMERA_FAR_PLANE,
        ..default()
    });

    bsn! {
        Camera3d
        Camera {
            is_active,
            clear_color: ClearColorConfig::Custom(CAMERA_CLEAR_COLOR),
        }
        template_value(projection)
    }
}

fn top_down_transform() -> Transform {
    Transform::from_xyz(0.0, 0.0, TOP_DOWN_CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y)
}

/// Reparents the over-the-shoulder camera to the active selected hero.
fn attach_over_shoulder_camera(
    mut commands: Commands,
    active_selected_hero: Single<Entity, ActiveSelectedHero>,
    over_shoulder_camera: Single<(Entity, Option<&ChildOf>), With<OverShoulderCamera>>,
) {
    let hero = active_selected_hero.into_inner();
    let (camera, child_of) = over_shoulder_camera.into_inner();
    let current_parent = child_of.map(ChildOf::parent);

    if current_parent != Some(hero) {
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
    use bevy::scene::ScenePlugin;

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
    fn camera_scene_spawns_both_cameras_with_the_requested_view_active() {
        for camera_view in [CameraView::Arena, CameraView::OverShoulder] {
            let mut app = camera_scene_test_app(camera_view);

            assert_camera::<TopDownCamera>(
                &mut app,
                matches!(camera_view, CameraView::Arena),
                TOP_DOWN_FIELD_OF_VIEW_RADIANS,
                top_down_transform(),
            );
            assert_camera::<OverShoulderCamera>(
                &mut app,
                matches!(camera_view, CameraView::OverShoulder),
                OVER_SHOULDER_FIELD_OF_VIEW_RADIANS,
                over_shoulder_local_transform(),
            );
        }
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
        app.insert_resource(CameraView::OverShoulder)
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

    fn camera_scene_test_app(camera_view: CameraView) -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            ScenePlugin,
        ))
        .insert_resource(camera_view)
        .add_systems(Startup, spawn_camera_scene);
        app.update();
        app
    }

    fn assert_camera<M: Component>(
        app: &mut App,
        expected_active: bool,
        expected_field_of_view_radians: f32,
        expected_transform: Transform,
    ) {
        let world = app.world_mut();
        let mut query =
            world.query_filtered::<(&Camera, &Projection, &Transform), (With<M>, With<Camera3d>)>();
        let (camera, projection, transform) = query
            .single(world)
            .expect("invariant: the camera scene spawns exactly one camera for this marker");
        let Projection::Perspective(projection) = projection else {
            panic!("invariant: each game camera uses a perspective projection");
        };

        assert_eq!(camera.is_active, expected_active);
        assert_eq!(projection.fov, expected_field_of_view_radians);
        assert_eq!(projection.near, CAMERA_NEAR_PLANE);
        assert_eq!(projection.far, CAMERA_FAR_PLANE);
        assert_eq!(transform, &expected_transform);
    }

    fn camera_parent(app: &App, camera: Entity) -> Entity {
        app.world()
            .get::<ChildOf>(camera)
            .expect("invariant: the shoulder camera is attached to a hero")
            .parent()
    }
}
