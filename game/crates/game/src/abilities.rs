use std::collections::HashMap;

use bevy::{ecs::schedule::common_conditions::on_message, prelude::*};

use crate::{
    enemy::Enemy,
    hero::Hero,
    movement::{CardinalDirection, HeroMoveBlockedByEnemy, HeroMovementSet, MovementLocked},
    tile,
};

/// Number of ability slots available to every hero.
pub const SLOT_COUNT: usize = 4;

const BASH_DURATION_SECONDS: f32 = 0.36;
const BASH_DISTANCE: f32 = tile::SIZE * 0.25;
const BASH_SOUND_PATH: &str = "abilities/bash.ogg";

type BashReadyHero = (With<Hero>, Without<BashAnimation>);
type AvailableBashTarget = (With<Enemy>, Without<BashTarget>);

/// Identifies an ability that can occupy one of a hero's ability slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Bash,
}

/// A shared lifecycle cue that an ability can emit when the phase applies.
///
/// These are deliberately lightweight notifications rather than authoritative
/// cast state. If abilities later need duration, cancellation, or overlapping
/// casts, an execution component can own that richer state and continue to emit
/// these phase notifications at its boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[expect(
    dead_code,
    reason = "Bash emits only Impact; Charge and Release are shared phases for future abilities"
)]
pub enum AbilityPhase {
    Charge,
    Release,
    Impact,
}

/// Announces that an ability reached one lifecycle phase.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityPhaseEntered {
    pub ability: Ability,
    pub phase: AbilityPhase,
    pub source: Entity,
    pub target: Option<Entity>,
    pub direction: Option<IVec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AbilitySoundKey {
    ability: Ability,
    phase: AbilityPhase,
}

impl AbilitySoundKey {
    const fn new(ability: Ability, phase: AbilityPhase) -> Self {
        Self { ability, phase }
    }
}

#[derive(Debug, Clone, Copy)]
struct AbilitySoundAsset {
    key: AbilitySoundKey,
    path: &'static str,
}

/// Central sound convention for ability lifecycle phases.
///
/// A single-phase ability may retain the concise `<ability>.ogg` filename, as
/// Bash does. When one ability has multiple sounds, use
/// `<ability>-<phase>.ogg`, such as `fireball-charge.ogg` and
/// `fireball-impact.ogg`. This table is the authoritative semantic mapping, so
/// playback never has to guess a phase from a filename.
const ABILITY_SOUND_ASSETS: &[AbilitySoundAsset] = &[AbilitySoundAsset {
    key: AbilitySoundKey::new(Ability::Bash, AbilityPhase::Impact),
    path: BASH_SOUND_PATH,
}];

/// The fixed-size ability loadout owned by a hero.
///
/// An array makes a fifth slot unrepresentable, while `None` represents an
/// intentionally empty slot.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Abilities {
    pub slots: [Option<Ability>; SLOT_COUNT],
}

impl Abilities {
    pub const fn new(slots: [Option<Ability>; SLOT_COUNT]) -> Self {
        Self { slots }
    }

    pub fn contains(&self, ability: Ability) -> bool {
        self.slots.contains(&Some(ability))
    }
}

/// Owns ability activation and presentation systems.
pub struct HeroAbilitiesPlugin;

impl Plugin for HeroAbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AbilityPhaseEntered>()
            .add_systems(Startup, load_ability_sounds)
            .add_systems(
                FixedUpdate,
                start_bashes
                    .after(HeroMovementSet::Apply)
                    .run_if(on_message::<HeroMoveBlockedByEnemy>),
            )
            .add_systems(
                Update,
                (
                    animate_bashes,
                    play_ability_sounds.run_if(on_message::<AbilityPhaseEntered>),
                ),
            );
    }
}

/// Handles for the ability sounds loaded once for reuse by short-lived players.
#[derive(Resource, Debug, Clone)]
struct AbilitySounds(HashMap<AbilitySoundKey, Handle<AudioSource>>);

/// One paired Bash animation, stored on its attacking hero.
#[derive(Component, Debug, Clone, Copy)]
struct BashAnimation {
    target: Entity,
    direction: Vec2,
    hero_origin: Vec3,
    target_origin: Vec3,
    elapsed_seconds: f32,
}

impl BashAnimation {
    fn new(
        target: Entity,
        direction: CardinalDirection,
        hero_origin: Vec3,
        target_origin: Vec3,
    ) -> Self {
        Self {
            target,
            direction: direction.tile_offset().as_vec2(),
            hero_origin,
            target_origin,
            elapsed_seconds: 0.0,
        }
    }

    fn advance(&mut self, delta_seconds: f32) -> f32 {
        self.elapsed_seconds = (self.elapsed_seconds + delta_seconds).min(BASH_DURATION_SECONDS);
        self.elapsed_seconds / BASH_DURATION_SECONDS
    }
}

/// Reserves an enemy so two Bashes cannot animate its transform at once.
#[derive(Component, Debug, Default, Clone, Copy)]
struct BashTarget;

fn load_ability_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut sounds = HashMap::with_capacity(ABILITY_SOUND_ASSETS.len());
    for asset in ABILITY_SOUND_ASSETS {
        let previous = sounds.insert(asset.key, asset_server.load(asset.path));
        assert!(
            previous.is_none(),
            "invariant: each ability phase has at most one sound"
        );
    }
    commands.insert_resource(AbilitySounds(sounds));
}

fn play_ability_sounds(
    mut entered_phases: MessageReader<AbilityPhaseEntered>,
    sounds: Option<Res<AbilitySounds>>,
    mut commands: Commands,
) {
    let Some(sounds) = sounds else {
        return;
    };

    for entered_phase in entered_phases.read() {
        let key = AbilitySoundKey::new(entered_phase.ability, entered_phase.phase);
        let Some(sound) = sounds.0.get(&key) else {
            continue;
        };

        commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
    }
}

fn start_bashes(
    mut blocked_moves: MessageReader<HeroMoveBlockedByEnemy>,
    heroes: Query<(&Abilities, &Transform), BashReadyHero>,
    enemies: Query<&Transform, AvailableBashTarget>,
    mut entered_phases: MessageWriter<AbilityPhaseEntered>,
    mut commands: Commands,
) {
    // Commands are deferred, so reserve participants locally as messages are
    // processed to keep this batch deterministic.
    let mut reserved_heroes = Vec::new();
    let mut reserved_targets = Vec::new();

    for blocked_move in blocked_moves.read() {
        if reserved_heroes.contains(&blocked_move.hero)
            || reserved_targets.contains(&blocked_move.enemy)
        {
            continue;
        }

        let Ok((abilities, hero_transform)) = heroes.get(blocked_move.hero) else {
            continue;
        };
        if !abilities.contains(Ability::Bash) {
            continue;
        }
        let Ok(target_transform) = enemies.get(blocked_move.enemy) else {
            continue;
        };

        reserved_heroes.push(blocked_move.hero);
        reserved_targets.push(blocked_move.enemy);
        entered_phases.write(AbilityPhaseEntered {
            ability: Ability::Bash,
            phase: AbilityPhase::Impact,
            source: blocked_move.hero,
            target: Some(blocked_move.enemy),
            direction: Some(blocked_move.direction.tile_offset()),
        });
        commands.entity(blocked_move.hero).insert((
            MovementLocked,
            BashAnimation::new(
                blocked_move.enemy,
                blocked_move.direction,
                hero_transform.translation,
                target_transform.translation,
            ),
        ));
        commands.entity(blocked_move.enemy).insert(BashTarget);
    }
}

fn animate_bashes(
    time: Res<Time>,
    mut animations: Query<(Entity, &mut BashAnimation)>,
    mut transforms: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (hero, mut animation) in &mut animations {
        let target = animation.target;
        let Ok([mut hero_transform, mut target_transform]) =
            transforms.get_many_mut([hero, target])
        else {
            if let Ok(mut hero_transform) = transforms.get_mut(hero) {
                hero_transform.translation = animation.hero_origin;
            }
            commands
                .entity(hero)
                .remove::<BashAnimation>()
                .remove::<MovementLocked>();
            if let Ok(mut target_commands) = commands.get_entity(target) {
                target_commands.remove::<BashTarget>();
            }
            continue;
        };

        let progress = animation.advance(time.delta_secs());
        let direction = animation.direction.extend(0.0) * BASH_DISTANCE;
        hero_transform.translation =
            animation.hero_origin + direction * attacker_displacement(progress);
        target_transform.translation =
            animation.target_origin + direction * target_displacement(progress);

        if progress < 1.0 {
            continue;
        }

        hero_transform.translation = animation.hero_origin;
        target_transform.translation = animation.target_origin;
        commands
            .entity(hero)
            .remove::<BashAnimation>()
            .remove::<MovementLocked>();
        if let Ok(mut target_commands) = commands.get_entity(target) {
            target_commands.remove::<BashTarget>();
        }
    }
}

fn attacker_displacement(progress: f32) -> f32 {
    pulse(progress, 0.0, 0.38, 0.85)
}

fn target_displacement(progress: f32) -> f32 {
    pulse(progress, 0.32, 0.62, 1.0)
}

fn pulse(progress: f32, start: f32, peak: f32, end: f32) -> f32 {
    debug_assert!(start < peak && peak < end);

    if progress <= start || progress >= end {
        0.0
    } else if progress <= peak {
        smoothstep((progress - start) / (peak - start))
    } else {
        1.0 - smoothstep((progress - peak) / (end - peak))
    }
}

fn smoothstep(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        hero::{self, Active, Selected},
        movement::{HeroMovementPlugin, MoveHero},
        tile::GridPosition,
    };

    #[test]
    fn default_loadout_has_exactly_four_empty_slots() {
        let abilities = Abilities::default();

        assert_eq!(abilities.slots, [None; SLOT_COUNT]);
    }

    #[test]
    fn bash_loadout_reports_the_equipped_ability() {
        let abilities = Abilities::new([Some(Ability::Bash), None, None, None]);

        assert!(abilities.contains(Ability::Bash));
    }

    #[test]
    fn bash_sound_uses_the_ogg_asset_contract() {
        let sound = include_bytes!("../assets/abilities/bash.ogg");

        assert!(sound.starts_with(b"OggS"));
        assert!(sound.len() > 4);
    }

    #[test]
    fn bash_registers_only_an_impact_sound() {
        let bash_phases = ABILITY_SOUND_ASSETS
            .iter()
            .filter(|asset| asset.key.ability == Ability::Bash)
            .map(|asset| asset.key.phase)
            .collect::<Vec<_>>();

        assert_eq!(bash_phases, vec![AbilityPhase::Impact]);
    }

    #[test]
    fn bash_profiles_lunge_then_knock_the_target_back() {
        assert_eq!(attacker_displacement(0.0), 0.0);
        assert_eq!(attacker_displacement(0.38), 1.0);
        assert!(target_displacement(0.38) > 0.0);
        assert_eq!(attacker_displacement(1.0), 0.0);
        assert_eq!(target_displacement(1.0), 0.0);
    }

    #[test]
    fn blocked_move_with_bash_animates_without_changing_grid_cells() {
        let mut app = bash_test_app();
        let hero_origin = Vec3::new(0.0, 0.0, hero::RADIUS);
        let target_grid_position = GridPosition(IVec2::Y);
        let target_origin = target_grid_position.world_xy().extend(tile::SIZE * 0.5);
        let hero = app
            .world_mut()
            .spawn((
                Hero,
                Selected,
                Active,
                Abilities::new([Some(Ability::Bash), None, None, None]),
                GridPosition::default(),
                Transform::from_translation(hero_origin),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                Enemy,
                target_grid_position,
                Transform::from_translation(target_origin),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<MoveHero>>()
            .write(MoveHero {
                direction: CardinalDirection::Up,
            });
        app.world_mut().run_schedule(FixedUpdate);

        assert_eq!(grid_position(&app, hero), GridPosition::default());
        assert_eq!(grid_position(&app, target), target_grid_position);
        assert_eq!(translation(&app, hero), hero_origin);
        assert_eq!(translation(&app, target), target_origin);
        assert!(app.world().get::<MovementLocked>(hero).is_some());
        assert!(app.world().get::<BashAnimation>(hero).is_some());
        let entered_phases = app.world().resource::<Messages<AbilityPhaseEntered>>();
        let mut phase_cursor = entered_phases.get_cursor();
        assert_eq!(
            phase_cursor
                .read(entered_phases)
                .copied()
                .collect::<Vec<_>>(),
            vec![AbilityPhaseEntered {
                ability: Ability::Bash,
                phase: AbilityPhase::Impact,
                source: hero,
                target: Some(target),
                direction: Some(IVec2::Y),
            }]
        );

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(BASH_DURATION_SECONDS * 0.5));
        app.world_mut().run_schedule(Update);

        assert!(translation(&app, hero).y > hero_origin.y);
        assert!(translation(&app, target).y > target_origin.y);
        assert_eq!(grid_position(&app, hero), GridPosition::default());
        assert_eq!(grid_position(&app, target), target_grid_position);
        assert_bash_sound_started(&mut app);

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(BASH_DURATION_SECONDS));
        app.world_mut().run_schedule(Update);

        assert_eq!(translation(&app, hero), hero_origin);
        assert_eq!(translation(&app, target), target_origin);
        assert!(app.world().get::<MovementLocked>(hero).is_none());
        assert!(app.world().get::<BashAnimation>(hero).is_none());
        assert!(app.world().get::<BashTarget>(target).is_none());
    }

    fn bash_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .add_plugins((HeroMovementPlugin, HeroAbilitiesPlugin))
            .insert_resource(AbilitySounds(HashMap::from([(
                AbilitySoundKey::new(Ability::Bash, AbilityPhase::Impact),
                Handle::default(),
            )])));
        app
    }

    fn assert_bash_sound_started(app: &mut App) {
        let world = app.world_mut();
        let mut players = world.query::<(&AudioPlayer, &PlaybackSettings)>();
        let (player, settings) = players
            .single(world)
            .expect("invariant: Bash starts exactly one sound");

        assert_eq!(player.0, Handle::<AudioSource>::default());
        assert!(matches!(settings.mode, bevy::audio::PlaybackMode::Despawn));
    }

    fn grid_position(app: &App, entity: Entity) -> GridPosition {
        *app.world()
            .get::<GridPosition>(entity)
            .expect("invariant: the tested character has a grid position")
    }

    fn translation(app: &App, entity: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(entity)
            .expect("invariant: the tested character has a transform")
            .translation
    }
}
