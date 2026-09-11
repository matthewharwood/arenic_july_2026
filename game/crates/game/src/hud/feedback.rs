//! Bounded stat updates and their transient, selected-character presentation.

use bevy::prelude::*;

use crate::roster::ArenaRoster;

use super::tokens;

const MAX_DELTAS_PER_TICK: usize = 64;
const MAX_POPUPS: usize = 8;
const POPUP_LIFETIME_SECONDS: f32 = 1.25;
const POPUP_TRAVEL_PIXELS: f32 = 30.0;

pub(super) struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CharacterDelta>()
            .add_message::<AppliedDelta>()
            .add_systems(
                FixedUpdate,
                apply_character_deltas.in_set(FeedbackSet::Apply),
            )
            .add_systems(
                Update,
                (
                    animate_feedback.run_if(any_with_component::<FloatingDelta>),
                    spawn_feedback,
                )
                    .chain()
                    .in_set(FeedbackSet::Present),
            );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum FeedbackSet {
    Apply,
    Present,
}

/// A permanent UI node outside the rebuilt selected-character card.
#[derive(Component)]
pub(super) struct FeedbackAnchor;

/// A resolved stat change, currently produced by the explicit HUD preview.
///
/// At most 64 messages are accepted per fixed tick; excess messages are consumed
/// and reported. A future combat producer must own its bounded queue and submit
/// resolved changes within this budget instead of relying on overflow delivery.
/// Experience accumulates without inventing level-up rules for this UI iteration.
#[derive(Message, Debug, Clone, Copy)]
pub(super) struct CharacterDelta {
    pub character_id: u32,
    pub experience: u32,
    pub hp: i32,
}

#[derive(Message, Clone, Copy)]
struct AppliedDelta {
    character_id: u32,
    experience: u32,
    hp: i64,
}

#[derive(Component)]
struct FloatingDelta {
    character_id: u32,
    elapsed_seconds: f32,
    color: Color,
    direction: f32,
    lane: usize,
}

fn apply_character_deltas(
    mut deltas: PopulatedMessageReader<CharacterDelta>,
    mut roster: ResMut<ArenaRoster>,
    mut feedback: MessageWriter<AppliedDelta>,
) {
    let mut unread = deltas.read();
    for delta in unread.by_ref().take(MAX_DELTAS_PER_TICK) {
        if let Some(applied) = apply_character_delta(&mut roster, *delta) {
            feedback.write(applied);
        }
    }
    let rejected = unread.count();
    if rejected > 0 {
        warn!(
            "Rejected {rejected} character deltas exceeding the per-tick limit of {MAX_DELTAS_PER_TICK}"
        );
    }
}

/// Unknown identities are rejected, and feedback reports the applied amount
/// after saturation rather than claiming gains or damage beyond the stat bounds.
fn apply_character_delta(roster: &mut ArenaRoster, delta: CharacterDelta) -> Option<AppliedDelta> {
    for arena in &mut roster.arenas {
        let Some(index) = arena
            .characters
            .iter()
            .position(|character| character.id == delta.character_id)
        else {
            continue;
        };
        let character = &mut arena.characters[index];
        let previous_experience = character.experience;
        let previous_hp = character.hp;
        character.experience = character.experience.saturating_add(delta.experience);
        character.hp = character
            .hp
            .saturating_add_signed(delta.hp)
            .min(character.max_hp);
        if character.hp == 0 && arena.selected == Some(index) {
            arena.selected = None;
        }
        return Some(AppliedDelta {
            character_id: delta.character_id,
            experience: character.experience.strict_sub(previous_experience),
            hp: i64::from(character.hp).strict_sub(i64::from(previous_hp)),
        });
    }
    None
}

fn spawn_feedback(
    mut commands: Commands,
    mut deltas: PopulatedMessageReader<AppliedDelta>,
    roster: Res<ArenaRoster>,
    anchor: Single<Entity, With<FeedbackAnchor>>,
    existing: Query<&FloatingDelta>,
) {
    let selected_id = roster.selected_character().map(|character| character.id);
    let mut occupied = [false; MAX_POPUPS];
    for popup in &existing {
        occupied[popup.lane] = true;
    }

    for delta in deltas.read() {
        if selected_id != Some(delta.character_id) {
            continue;
        }
        let amounts = [
            (delta.experience > 0)
                .then(|| (format!("+{} XP", delta.experience), tokens::XP, -1.0, 0.0)),
            (delta.hp != 0).then(|| {
                (
                    format!("{:+} HP", delta.hp),
                    if delta.hp > 0 {
                        tokens::POSITIVE
                    } else {
                        tokens::NEGATIVE
                    },
                    if delta.hp > 0 { -1.0 } else { 1.0 },
                    20.0,
                )
            }),
        ];
        for (label, color, direction, top) in amounts.into_iter().flatten() {
            let Some(lane) = occupied.iter().position(|is_used| !is_used) else {
                // Shed only transient presentation; authoritative changes have
                // already applied. The entire message stream is still consumed.
                continue;
            };
            occupied[lane] = true;
            commands.spawn((
                Text::new(label),
                TextFont::from_font_size(tokens::BODY),
                TextColor(color),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px((lane / 2) as f32 * 18.0),
                    top: Val::Px(top),
                    ..default()
                },
                UiTransform::default(),
                FloatingDelta {
                    character_id: delta.character_id,
                    elapsed_seconds: 0.0,
                    color,
                    direction,
                    lane,
                },
                ChildOf(*anchor),
            ));
        }
    }
}

fn animate_feedback(
    mut commands: Commands,
    time: Res<Time>,
    roster: Res<ArenaRoster>,
    mut popups: Query<(Entity, &mut FloatingDelta, &mut UiTransform, &mut TextColor)>,
) {
    let selected_id = roster.selected_character().map(|character| character.id);
    for (entity, mut popup, mut transform, mut color) in &mut popups {
        popup.elapsed_seconds += time.delta_secs();
        if popup.elapsed_seconds >= POPUP_LIFETIME_SECONDS
            || selected_id != Some(popup.character_id)
        {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = popup.elapsed_seconds / POPUP_LIFETIME_SECONDS;
        transform.translation = Val2::px(0.0, popup.direction * POPUP_TRAVEL_PIXELS * progress);
        color.0 = popup.color.with_alpha(1.0 - progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn feedback_app() -> App {
        let mut app = App::new();
        app.init_resource::<ArenaRoster>()
            .init_resource::<Time>()
            .add_plugins(FeedbackPlugin);
        app.world_mut().spawn((Node::default(), FeedbackAnchor));
        app
    }

    fn write_delta(app: &mut App, delta: CharacterDelta) {
        app.world_mut()
            .resource_mut::<Messages<CharacterDelta>>()
            .write(delta);
    }

    fn apply_and_present(app: &mut App) {
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().run_schedule(Update);
    }

    fn popup_count(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query::<&FloatingDelta>().iter(world).count()
    }

    #[test]
    fn messages_apply_bounded_stats_without_inventing_level_progression() {
        let mut app = feedback_app();
        write_delta(
            &mut app,
            CharacterDelta {
                character_id: 0,
                experience: u32::MAX,
                hp: i32::MAX,
            },
        );
        apply_and_present(&mut app);

        let character = &app.world().resource::<ArenaRoster>().arenas[0].characters[0];
        assert_eq!(character.experience, u32::MAX);
        assert_eq!(character.hp, character.max_hp);
        assert_eq!(character.level, 10);
        let world = app.world_mut();
        let labels: Vec<_> = world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.clone())
            .collect();
        assert!(labels.contains(&"+4294966675 XP".to_owned()));
        assert!(labels.contains(&"+1001 HP".to_owned()));
    }

    #[test]
    fn lethal_damage_clears_only_that_arenas_selection() {
        let mut roster = ArenaRoster::default();
        let result = apply_character_delta(
            &mut roster,
            CharacterDelta {
                character_id: 100,
                experience: 0,
                hp: i32::MIN,
            },
        )
        .expect("invariant: fixture character 100 exists");

        assert_eq!(roster.arenas[1].characters[0].hp, 0);
        assert_eq!(roster.arenas[1].selected, None);
        assert_eq!(roster.arenas[0].selected, Some(0));
        assert_eq!(result.hp, -1_700);
    }

    #[test]
    fn unknown_identity_is_rejected_without_stat_changes() {
        let mut roster = ArenaRoster::default();
        assert!(
            apply_character_delta(
                &mut roster,
                CharacterDelta {
                    character_id: u32::MAX,
                    experience: 9,
                    hp: -9,
                },
            )
            .is_none()
        );
        let selected = roster
            .selected_character()
            .expect("invariant: selected fixture is alive");
        assert_eq!(selected.hp, 999);
        assert_eq!(selected.experience, 620);
    }

    #[test]
    fn visual_pressure_sheds_popups_without_losing_accepted_stat_changes() {
        let mut app = feedback_app();
        for _ in 0..MAX_DELTAS_PER_TICK.strict_add(1) {
            write_delta(
                &mut app,
                CharacterDelta {
                    character_id: 0,
                    experience: 1,
                    hp: -1,
                },
            );
        }
        apply_and_present(&mut app);

        assert_eq!(popup_count(&mut app), MAX_POPUPS);
        let character = &app.world().resource::<ArenaRoster>().arenas[0].characters[0];
        assert_eq!(character.experience, 684);
        assert_eq!(character.hp, 935);

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(2));
        apply_and_present(&mut app);
        assert_eq!(popup_count(&mut app), 0);
        let character = &app.world().resource::<ArenaRoster>().arenas[0].characters[0];
        assert_eq!(
            character.experience, 684,
            "rejected deltas must not replay later"
        );
    }

    #[test]
    fn inactive_arena_updates_apply_without_visible_feedback() {
        let mut app = feedback_app();
        write_delta(
            &mut app,
            CharacterDelta {
                character_id: 100,
                experience: 9,
                hp: -9,
            },
        );
        apply_and_present(&mut app);

        assert_eq!(popup_count(&mut app), 0);
        let character = &app.world().resource::<ArenaRoster>().arenas[1].characters[0];
        assert_eq!(character.experience, 309);
        assert_eq!(character.hp, 1_691);
    }

    #[test]
    fn feedback_moves_in_semantic_directions_then_clears_on_selection_change() {
        let mut app = feedback_app();
        write_delta(
            &mut app,
            CharacterDelta {
                character_id: 0,
                experience: 9,
                hp: -9,
            },
        );
        apply_and_present(&mut app);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.5));
        app.world_mut().run_schedule(Update);

        let world = app.world_mut();
        for (text, transform, color) in world
            .query::<(&Text, &UiTransform, &TextColor)>()
            .iter(world)
        {
            let Val::Px(y) = transform.translation.y else {
                panic!("invariant: feedback uses pixel translation");
            };
            if text.0.ends_with("XP") {
                assert!(y < 0.0);
            } else {
                assert!(y > 0.0);
            }
            assert!((color.0.alpha() - 0.6).abs() < 0.001);
        }
        assert!(
            app.world_mut()
                .resource_mut::<ArenaRoster>()
                .select_character(1)
        );
        app.world_mut().run_schedule(Update);
        assert_eq!(popup_count(&mut app), 0);
    }
}
