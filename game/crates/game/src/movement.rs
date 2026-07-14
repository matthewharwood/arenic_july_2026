use bevy::{ecs::schedule::common_conditions::on_message, prelude::*};

use crate::{
    hero::{Active, Hero, Selected},
    tile,
};

type ControllableHero = (With<Hero>, With<Selected>, With<Active>);

/// Adds device input and tile-based hero movement.
pub struct HeroMovementPlugin;

impl Plugin for HeroMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveHero>().add_systems(
            Update,
            (
                read_gamepad_movement,
                apply_hero_movement.run_if(on_message::<MoveHero>),
            )
                .chain(),
        );
    }
}

/// A hardware-independent request to move the active, selected hero.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveHero(pub CardinalDirection);

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

fn read_gamepad_movement(gamepads: Query<&Gamepad>, mut moves: MessageWriter<MoveHero>) {
    for gamepad in &gamepads {
        if let Some(direction) = direction_for_pressed_dpad(gamepad) {
            moves.write(MoveHero(direction));
            return;
        }
    }
}

fn direction_for_pressed_dpad(gamepad: &Gamepad) -> Option<CardinalDirection> {
    [
        GamepadButton::DPadUp,
        GamepadButton::DPadDown,
        GamepadButton::DPadLeft,
        GamepadButton::DPadRight,
    ]
    .into_iter()
    .find(|button| gamepad.just_pressed(*button))
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
    let mut direction = None;

    for movement in moves.read() {
        direction.get_or_insert(movement.0);
    }

    let Some(direction) = direction else {
        return;
    };

    let offset = direction.tile_offset().as_vec2() * tile::SIZE;
    for mut transform in &mut heroes {
        transform.translation.x += offset.x;
        transform.translation.y += offset.y;
    }
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
    fn dpad_press_moves_the_hero_one_tile() {
        let mut app = movement_test_app();
        let hero = app
            .world_mut()
            .spawn((Hero, Selected, Active, Transform::default()))
            .id();
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();

        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .expect("invariant: the test gamepad entity has a Gamepad component")
            .digital_mut()
            .press(GamepadButton::DPadDown);
        app.update();

        assert_eq!(hero_position(&app, hero), Vec3::new(0.0, -tile::SIZE, 0.0));
    }

    fn movement_test_app() -> App {
        let mut app = App::new();
        app.add_message::<MoveHero>().add_systems(
            Update,
            (
                read_gamepad_movement,
                apply_hero_movement.run_if(on_message::<MoveHero>),
            )
                .chain(),
        );
        app
    }

    fn hero_position(app: &App, hero: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(hero)
            .expect("invariant: the test hero entity has a Transform component")
            .translation
    }
}
