use bevy::{ecs::schedule::common_conditions::on_message, prelude::*};

use crate::{
    hero::{Active, Hero, Selected},
    hud::HudSelectionSet,
    tile,
};

type ControllableHero = (With<Hero>, With<Selected>, With<Active>);

/// Resolves conflicting presses independently of gamepad query order.
const DPAD_BUTTON_PRIORITY: [GamepadButton; 4] = [
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
];

/// Adds device input and tile-based hero movement.
pub struct HeroMovementPlugin;

impl Plugin for HeroMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveHero>()
            .add_systems(Update, read_gamepad_movement.after(HudSelectionSet))
            .add_systems(
                FixedUpdate,
                apply_hero_movement.run_if(on_message::<MoveHero>),
            );
    }
}

/// A hardware-independent request to move the active, selected hero.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveHero {
    pub(crate) direction: CardinalDirection,
}

/// A single tile-aligned movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardinalDirection {
    Up,
    Down,
    Left,
    Right,
}

impl CardinalDirection {
    const fn tile_offset(self) -> IVec2 {
        match self {
            Self::Up => IVec2::Y,
            Self::Down => IVec2::NEG_Y,
            Self::Left => IVec2::NEG_X,
            Self::Right => IVec2::X,
        }
    }
}

fn read_gamepad_movement(
    gamepads: Query<&Gamepad>,
    heroes: Query<(), ControllableHero>,
    mut moves: MessageWriter<MoveHero>,
) {
    // The selection set's deferred commands finish before this adapter, so
    // entering an empty arena cannot enqueue input for a previous selection.
    if heroes.single().is_ok()
        && let Some(direction) = direction_for_pressed_dpad(&gamepads)
    {
        moves.write(MoveHero { direction });
    }
}

fn direction_for_pressed_dpad(gamepads: &Query<&Gamepad>) -> Option<CardinalDirection> {
    DPAD_BUTTON_PRIORITY
        .into_iter()
        .find(|button| gamepads.iter().any(|gamepad| gamepad.just_pressed(*button)))
        .and_then(direction_for_button)
}

const fn direction_for_button(button: GamepadButton) -> Option<CardinalDirection> {
    match button {
        GamepadButton::DPadUp => Some(CardinalDirection::Up),
        GamepadButton::DPadDown => Some(CardinalDirection::Down),
        GamepadButton::DPadLeft => Some(CardinalDirection::Left),
        GamepadButton::DPadRight => Some(CardinalDirection::Right),
        _ => None,
    }
}

fn apply_hero_movement(
    mut moves: MessageReader<MoveHero>,
    mut heroes: Query<&mut Transform, ControllableHero>,
) {
    let direction = moves.read().next().map(|movement| movement.direction);
    // Consume every queued intent even when no hero is active. Only the first
    // intent may move a hero in this fixed step; nothing replays after selection.
    moves.clear();
    let Some(direction) = direction else {
        return;
    };
    let Ok(mut hero) = heroes.single_mut() else {
        return;
    };

    let offset = direction.tile_offset().as_vec2() * tile::SIZE;
    hero.translation.x += offset.x;
    hero.translation.y += offset.y;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_direction_moves_along_exactly_one_axis() {
        for direction in [
            CardinalDirection::Up,
            CardinalDirection::Down,
            CardinalDirection::Left,
            CardinalDirection::Right,
        ] {
            let offset = direction.tile_offset();
            assert_eq!(offset.x.abs() + offset.y.abs(), 1);
        }
    }

    #[test]
    fn dpad_buttons_map_to_cardinal_directions() {
        for (button, direction) in [
            (GamepadButton::DPadUp, CardinalDirection::Up),
            (GamepadButton::DPadDown, CardinalDirection::Down),
            (GamepadButton::DPadLeft, CardinalDirection::Left),
            (GamepadButton::DPadRight, CardinalDirection::Right),
        ] {
            assert_eq!(direction_for_button(button), Some(direction));
        }
    }

    #[test]
    fn non_dpad_buttons_do_not_map_to_cardinal_directions() {
        assert_eq!(direction_for_button(GamepadButton::South), None);
    }

    #[test]
    fn dpad_press_moves_only_the_active_selected_hero_one_tile() {
        let mut app = movement_test_app();
        let hero = app
            .world_mut()
            .spawn((Hero, Selected, Active, Transform::default()))
            .id();
        let inactive_hero = app
            .world_mut()
            .spawn((Hero, Selected, Transform::default()))
            .id();
        let unselected_hero = app
            .world_mut()
            .spawn((Hero, Active, Transform::default()))
            .id();
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();

        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::DPadDown);
        app.update();
        app.world_mut().run_schedule(FixedUpdate);

        assert_eq!(hero_position(&app, hero), Vec3::new(0.0, -tile::SIZE, 0.0));
        assert_eq!(hero_position(&app, inactive_hero), Vec3::ZERO);
        assert_eq!(hero_position(&app, unselected_hero), Vec3::ZERO);
    }

    #[test]
    fn simultaneous_dpad_presses_follow_fixed_priority_across_gamepads() {
        for buttons in [
            [GamepadButton::DPadRight, GamepadButton::DPadUp],
            [GamepadButton::DPadUp, GamepadButton::DPadRight],
        ] {
            let mut app = movement_test_app();
            let hero = app
                .world_mut()
                .spawn((Hero, Selected, Active, Transform::default()))
                .id();

            for button in buttons {
                let gamepad = app.world_mut().spawn(Gamepad::default()).id();
                app.world_mut()
                    .get_mut::<Gamepad>(gamepad)
                    .expect("invariant: the test gamepad entity has a Gamepad component")
                    .digital_mut()
                    .press(button);
            }

            app.update();
            app.world_mut().run_schedule(FixedUpdate);

            assert_eq!(hero_position(&app, hero), Vec3::new(0.0, tile::SIZE, 0.0));
        }
    }

    fn movement_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(HeroMovementPlugin);
        app
    }

    #[derive(Resource)]
    struct TestSelection {
        hero: Entity,
        active: bool,
    }

    fn sync_test_selection(
        mut commands: Commands,
        selection: Res<TestSelection>,
        mut heroes: Query<&mut Transform, With<Hero>>,
    ) {
        if !selection.is_changed() {
            return;
        }
        if selection.active {
            let mut transform = heroes
                .get_mut(selection.hero)
                .expect("invariant: the test selection refers to a hero");
            transform.translation = Vec3::ZERO;
            commands.entity(selection.hero).insert(Active);
        } else {
            commands.entity(selection.hero).remove::<Active>();
        }
    }

    #[test]
    fn simultaneous_selection_and_dpad_input_wait_for_the_fixed_step() {
        let mut app = movement_test_app();
        let hero = app
            .world_mut()
            .spawn((Hero, Selected, Transform::from_xyz(3.0, 0.0, 0.0)))
            .id();
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();
        app.insert_resource(TestSelection { hero, active: true })
            .add_systems(Update, sync_test_selection.in_set(HudSelectionSet));
        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::DPadRight);

        app.update();

        assert_eq!(hero_position(&app, hero), Vec3::ZERO);
        assert!(app.world().get::<Active>(hero).is_some());
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(hero_position(&app, hero), Vec3::new(tile::SIZE, 0.0, 0.0));
    }

    #[test]
    fn empty_arena_input_is_rejected_and_pending_intents_do_not_replay() {
        let mut app = movement_test_app();
        let hero = app
            .world_mut()
            .spawn((Hero, Selected, Active, Transform::default()))
            .id();
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();
        app.insert_resource(TestSelection {
            hero,
            active: false,
        })
        .add_systems(Update, sync_test_selection.in_set(HudSelectionSet));
        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::DPadRight);

        app.update();

        assert!(app.world().get::<Active>(hero).is_none());
        assert!(app.world().resource::<Messages<MoveHero>>().is_empty());
        // Exercise the consumer independently of the hardware gate: an intent
        // already in flight must be consumed while the selection is empty.
        app.world_mut()
            .resource_mut::<Messages<MoveHero>>()
            .write(MoveHero {
                direction: CardinalDirection::Right,
            });
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().resource_mut::<TestSelection>().active = true;
        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .reset_all();
        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::DPadUp);
        app.update();
        assert_eq!(hero_position(&app, hero), Vec3::ZERO);
        app.world_mut().run_schedule(FixedUpdate);

        assert!(app.world().get::<Active>(hero).is_some());
        assert_eq!(hero_position(&app, hero), Vec3::new(0.0, tile::SIZE, 0.0));
    }

    fn hero_position(app: &App, hero: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(hero)
            .expect("invariant: the test hero entity has a Transform component")
            .translation
    }
}
