use bevy::{ecs::schedule::common_conditions::on_message, prelude::*};

use crate::{
    enemy::Enemy,
    hero::{Active, Hero, Selected},
    tile::{GridActor, GridPosition},
};

type ControllableHero = (
    With<Hero>,
    With<Selected>,
    With<Active>,
    Without<Enemy>,
    Without<MovementLocked>,
);
type EnemyOnly = (With<Enemy>, Without<Hero>);

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
            .add_message::<HeroMoveBlockedByEnemy>()
            .add_systems(Update, read_gamepad_movement)
            .add_systems(
                FixedUpdate,
                (
                    initialize_added_grid_actors,
                    apply_hero_movement
                        .in_set(HeroMovementSet::Apply)
                        .run_if(on_message::<MoveHero>),
                )
                    .chain(),
            );
    }
}

/// Ordering boundary for rules that react to attempted hero movement.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HeroMovementSet {
    Apply,
}

/// Temporarily prevents a hero from accepting another movement intent.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct MovementLocked;

/// A hardware-independent request to move the active, selected hero.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveHero {
    pub(crate) direction: CardinalDirection,
}

/// Reports that a hero's requested destination is occupied by one enemy.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeroMoveBlockedByEnemy {
    pub(crate) hero: Entity,
    pub(crate) enemy: Entity,
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
    pub(crate) const fn tile_offset(self) -> IVec2 {
        match self {
            Self::Up => IVec2::Y,
            Self::Down => IVec2::NEG_Y,
            Self::Left => IVec2::NEG_X,
            Self::Right => IVec2::X,
        }
    }
}

fn read_gamepad_movement(gamepads: Query<&Gamepad>, mut moves: MessageWriter<MoveHero>) {
    if let Some(direction) = direction_for_pressed_dpad(&gamepads) {
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

fn initialize_added_grid_actors(
    mut actors: Query<(Entity, Option<&GridPosition>, &mut Transform), Added<GridActor>>,
    mut commands: Commands,
) {
    for (entity, grid_position, mut transform) in &mut actors {
        let grid_position = if let Some(grid_position) = grid_position {
            *grid_position
        } else {
            let Some(grid_position) = GridPosition::from_world_xy(transform.translation.truncate())
            else {
                error!(
                    "cannot initialize grid actor {entity:?} from non-finite or out-of-range translation {:?}",
                    transform.translation
                );
                continue;
            };
            commands.entity(entity).insert(grid_position);
            grid_position
        };

        transform.translation = grid_position.world_xy().extend(transform.translation.z);
    }
}

fn apply_hero_movement(
    mut moves: MessageReader<MoveHero>,
    mut heroes: Query<(Entity, &mut GridPosition, &mut Transform), ControllableHero>,
    enemies: Query<(Entity, &GridPosition), EnemyOnly>,
    mut blocked_moves: MessageWriter<HeroMoveBlockedByEnemy>,
) {
    let mut direction = None;

    for movement in moves.read() {
        direction.get_or_insert(movement.direction);
    }

    let Some(direction) = direction else {
        return;
    };

    let Ok((hero, mut grid_position, mut transform)) = heroes.single_mut() else {
        return;
    };
    let Some(destination) = grid_position.checked_offset(direction.tile_offset()) else {
        return;
    };

    let mut enemy_at_destination = None;
    for (enemy, enemy_position) in &enemies {
        if *enemy_position != destination {
            continue;
        }

        // Multiple enemies in one grid cell is invalid and has no deterministic
        // collision target, so reject the move without selecting either one.
        if enemy_at_destination.replace(enemy).is_some() {
            return;
        }
    }

    if let Some(enemy) = enemy_at_destination {
        blocked_moves.write(HeroMoveBlockedByEnemy {
            hero,
            enemy,
            direction,
        });
        return;
    }

    *grid_position = destination;
    transform.translation = destination.world_xy().extend(transform.translation.z);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile;

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
        run_input_and_simulation(&mut app);

        assert_eq!(hero_position(&app, hero), Vec3::new(0.0, -tile::SIZE, 0.0));
        assert_eq!(hero_grid_position(&app, hero), GridPosition(IVec2::NEG_Y));
        assert_eq!(hero_position(&app, inactive_hero), Vec3::ZERO);
        assert_eq!(
            hero_grid_position(&app, inactive_hero),
            GridPosition::default()
        );
        assert_eq!(hero_position(&app, unselected_hero), Vec3::ZERO);
        assert_eq!(
            hero_grid_position(&app, unselected_hero),
            GridPosition::default()
        );
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

            run_input_and_simulation(&mut app);

            assert_eq!(hero_position(&app, hero), Vec3::new(0.0, tile::SIZE, 0.0));
            assert_eq!(hero_grid_position(&app, hero), GridPosition(IVec2::Y));
        }
    }

    #[test]
    fn transform_only_enemy_derives_grid_position_before_collision() {
        let mut app = movement_test_app();
        let hero = app
            .world_mut()
            .spawn((Hero, Selected, Active, Transform::default()))
            .id();
        let enemy_grid_position = GridPosition(IVec2::Y);
        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                Transform::from_translation(enemy_grid_position.world_xy().extend(tile::SIZE)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<MoveHero>>()
            .write(MoveHero {
                direction: CardinalDirection::Up,
            });
        app.world_mut().run_schedule(FixedUpdate);

        assert_eq!(hero_grid_position(&app, hero), GridPosition::default());
        assert_eq!(hero_grid_position(&app, enemy), enemy_grid_position);
        let blocked_moves = app.world().resource::<Messages<HeroMoveBlockedByEnemy>>();
        let mut cursor = blocked_moves.get_cursor();
        assert_eq!(
            cursor.read(blocked_moves).copied().collect::<Vec<_>>(),
            vec![HeroMoveBlockedByEnemy {
                hero,
                enemy,
                direction: CardinalDirection::Up,
            }]
        );
    }

    #[test]
    fn explicit_grid_position_corrects_a_mismatched_spawn_transform() {
        let mut app = movement_test_app();
        let grid_position = GridPosition(IVec2::new(-2, 3));
        let z = tile::SIZE;
        let enemy = app
            .world_mut()
            .spawn((Enemy, grid_position, Transform::from_xyz(99.0, 99.0, z)))
            .id();

        app.world_mut().run_schedule(FixedUpdate);

        assert_eq!(hero_grid_position(&app, enemy), grid_position);
        assert_eq!(
            hero_position(&app, enemy),
            grid_position.world_xy().extend(z)
        );
    }

    fn movement_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(HeroMovementPlugin);
        app
    }

    fn run_input_and_simulation(app: &mut App) {
        app.world_mut().run_schedule(Update);
        app.world_mut().run_schedule(FixedUpdate);
    }

    fn hero_position(app: &App, hero: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(hero)
            .expect("invariant: the test hero entity has a Transform component")
            .translation
    }

    fn hero_grid_position(app: &App, hero: Entity) -> GridPosition {
        *app.world()
            .get::<GridPosition>(hero)
            .expect("invariant: the test hero entity has a GridPosition component")
    }
}
