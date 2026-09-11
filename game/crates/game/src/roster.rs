//! Renderer-independent state for the first HUD iteration.
//!
//! The finite fixture has no spawning or storage producer. Forty is a visible
//! roster capacity, not a destructive limit on the character collection.

mod names;

use bevy::prelude::*;

use names::character_name;

pub const ARENA_COUNT: usize = 9;
pub const VISIBLE_ROSTER_SLOTS: usize = 40;

/// Owns roster state and effect expiry independently of the rendered HUD.
pub struct RosterPlugin;

impl Plugin for RosterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArenaRoster>()
            .init_resource::<StatusEffectClock>()
            .add_systems(
                FixedUpdate,
                advance_status_effects.run_if(any_active_effects),
            );
    }
}

#[derive(Resource)]
struct StatusEffectClock(Timer);

impl Default for StatusEffectClock {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn any_active_effects(roster: Res<ArenaRoster>) -> bool {
    roster.arenas.iter().any(|arena| {
        arena
            .characters
            .iter()
            .any(|character| !character.effects.is_empty())
    })
}

fn advance_status_effects(
    time: Res<Time<Fixed>>,
    mut clock: ResMut<StatusEffectClock>,
    mut roster: ResMut<ArenaRoster>,
) {
    if clock.0.tick(time.delta()).just_finished() {
        advance_effects_one_second(&mut roster);
    }
}

fn advance_effects_one_second(roster: &mut ArenaRoster) {
    // Effects keep aging in every arena, including characters outside selection.
    // The finite fixture only shrinks; there is no effect-spawning producer yet.
    for arena in &mut roster.arenas {
        for character in &mut arena.characters {
            character.effects.retain_mut(|effect| {
                effect.remaining_seconds = effect.remaining_seconds.saturating_sub(1);
                effect.remaining_seconds > 0
            });
        }
    }
}

#[derive(Resource, Debug)]
pub struct ArenaRoster {
    pub arenas: [Arena; ARENA_COUNT],
    pub active_arena: usize,
}

#[derive(Debug)]
pub struct Arena {
    pub name: &'static str,
    pub characters: Vec<Character>,
    pub selected: Option<usize>,
    pub alert: bool,
}

#[derive(Debug)]
pub struct Character {
    pub id: u32,
    pub name: String,
    pub class: CharacterClass,
    pub level: u16,
    pub experience: u32,
    pub next_level_experience: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub effects: Vec<StatusEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Warrior,
    Hunter,
    Healer,
    Mage,
    Guardian,
    Bard,
}

impl CharacterClass {
    pub const fn initial(self) -> &'static str {
        match self {
            Self::Warrior => "W",
            Self::Hunter | Self::Healer => "H",
            Self::Mage => "M",
            Self::Guardian => "G",
            Self::Bard => "B",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Warrior => "Warrior",
            Self::Hunter => "Hunter",
            Self::Healer => "Healer",
            Self::Mage => "Mage",
            Self::Guardian => "Guardian",
            Self::Bard => "Bard",
        }
    }

    pub const fn abilities(self) -> [&'static str; 4] {
        match self {
            Self::Warrior => ["Block", "Bash", "Taunt", "Barrier"],
            Self::Hunter => ["Shoot", "Volley", "Snare", "Mark"],
            Self::Healer => ["Mend", "Renew", "Cleanse", "Revive"],
            Self::Mage => ["Spark", "Frost", "Blink", "Nova"],
            Self::Guardian => ["Guard", "Slam", "Rally", "Aegis"],
            Self::Bard => ["Chord", "Anthem", "Lullaby", "Encore"],
        }
    }
}

#[derive(Debug)]
pub struct StatusEffect {
    pub name: &'static str,
    pub beneficial: bool,
    pub remaining_seconds: u16,
    pub stacks: u8,
}

impl Character {
    /// Allocates a display view with urgent debuffs first, then soonest expiry.
    /// A name tie-break keeps equal-duration tags stable between refreshes.
    pub fn sorted_effects(&self) -> Vec<&StatusEffect> {
        let mut effects: Vec<_> = self.effects.iter().collect();
        effects.sort_by_key(|effect| (effect.beneficial, effect.remaining_seconds, effect.name));
        effects
    }
}

impl ArenaRoster {
    pub fn current_arena(&self) -> &Arena {
        &self.arenas[self.active_arena]
    }

    pub fn selected_character(&self) -> Option<&Character> {
        let arena = self.current_arena();
        arena
            .characters
            .get(arena.selected?)
            .filter(|character| character.hp > 0)
    }

    /// Accepts only an existing arena and preserves each arena's selection.
    pub fn select_arena(&mut self, index: usize) -> bool {
        if index >= ARENA_COUNT {
            return false;
        }
        self.active_arena = index;
        true
    }

    pub fn cycle_arena(&mut self, delta: i32) {
        let arena_count =
            i32::try_from(ARENA_COUNT).expect("invariant: nine arena indices fit in an i32");
        let offset = usize::try_from(delta.rem_euclid(arena_count))
            .expect("invariant: wrapped arena offset is nonnegative");
        self.active_arena = self.active_arena.strict_add(offset) % ARENA_COUNT;
    }

    /// Overflow characters remain selectable; dead and empty slots do not.
    pub fn select_character(&mut self, index: usize) -> bool {
        let arena = &mut self.arenas[self.active_arena];
        if arena
            .characters
            .get(index)
            .is_none_or(|character| character.hp == 0)
        {
            return false;
        }
        arena.selected = Some(index);
        true
    }

    pub fn cycle_character(&mut self, reverse: bool) {
        let arena = &mut self.arenas[self.active_arena];
        let count = arena.characters.len();
        if count == 0 {
            arena.selected = None;
            return;
        }

        let selected = arena.selected.filter(|index| *index < count);
        // Each search visits at most the stored roster length, including overflow.
        arena.selected = if reverse {
            let start = selected.unwrap_or(count);
            (0..start)
                .rev()
                .chain((start..count).rev())
                .find(|index| arena.characters[*index].hp > 0)
        } else {
            let start = selected.map_or(0, |index| index.strict_add(1));
            (start..count)
                .chain(0..start)
                .find(|index| arena.characters[*index].hp > 0)
        };
    }
}

impl Default for ArenaRoster {
    fn default() -> Self {
        let arena_names = [
            "The Labyrinth",
            "Ashfall Keep",
            "Hollow Grove",
            "Sunken Archive",
            "Iron Hollow",
            "Glass Spire",
            "Silent Marsh",
            "Ember Crucible",
            "Moonwell",
        ];
        let character_counts = [41, 7, 0, 4, 0, 3, 0, 2, 0];
        Self {
            arenas: std::array::from_fn(|arena_index| {
                let identity_offset = u32::try_from(arena_index)
                    .expect("invariant: nine arena indices fit in a u32")
                    .strict_mul(100);
                let characters: Vec<_> = (0..character_counts[arena_index])
                    .map(|index| fixture_character(identity_offset.strict_add(index)))
                    .collect();
                let selected = characters.iter().position(|character| character.hp > 0);
                Arena {
                    name: arena_names[arena_index],
                    characters,
                    selected,
                    alert: arena_index == 5,
                }
            }),
            active_arena: 0,
        }
    }
}

fn fixture_character(id: u32) -> Character {
    let classes = [
        CharacterClass::Warrior,
        CharacterClass::Hunter,
        CharacterClass::Healer,
        CharacterClass::Mage,
        CharacterClass::Guardian,
        CharacterClass::Bard,
    ];
    let class_index =
        usize::try_from(id % 6).expect("invariant: character class index is below six");
    Character {
        id,
        name: character_name(id),
        class: classes[class_index],
        level: if id == 0 {
            10
        } else {
            u16::try_from((id % 15).strict_add(1))
                .expect("invariant: prototype character level is at most 15")
        },
        experience: if id == 0 {
            620
        } else {
            id.strict_mul(73) % 1_000
        },
        next_level_experience: 1_000,
        hp: if id != 0 && id.is_multiple_of(11) {
            0
        } else if id == 0 {
            999
        } else {
            1_200_u32.strict_add(id.strict_mul(29) % 800)
        },
        max_hp: 2_000,
        effects: if id == 0 {
            vec![
                StatusEffect {
                    name: "Bleed",
                    beneficial: false,
                    remaining_seconds: 39,
                    stacks: 1,
                },
                StatusEffect {
                    name: "Regain",
                    beneficial: true,
                    remaining_seconds: 39,
                    stacks: 1,
                },
                StatusEffect {
                    name: "Frostbite",
                    beneficial: false,
                    remaining_seconds: 3,
                    stacks: 2,
                },
                StatusEffect {
                    name: "Haste",
                    beneficial: true,
                    remaining_seconds: 12,
                    stacks: 1,
                },
            ]
        } else {
            Vec::new()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn effects_expire_without_changing_stacks_in_inactive_arenas() {
        let mut roster = ArenaRoster::default();
        assert!(roster.select_arena(2));
        roster.arenas[0].characters[0].effects[2].remaining_seconds = 1;
        roster.arenas[0].characters[0].effects[0].stacks = 3;

        advance_effects_one_second(&mut roster);

        let effects = &roster.arenas[0].characters[0].effects;
        assert_eq!(effects.len(), 3);
        assert!(effects.iter().all(|effect| effect.name != "Frostbite"));
        assert_eq!(effects[0].name, "Bleed");
        assert_eq!(effects[0].remaining_seconds, 38);
        assert_eq!(effects[0].stacks, 3);
        assert_eq!(effects[1].name, "Regain");
        assert_eq!(effects[1].remaining_seconds, 38);
        assert_eq!(effects[1].stacks, 1);

        for _ in 0..38 {
            advance_effects_one_second(&mut roster);
        }
        assert!(roster.arenas[0].characters[0].effects.is_empty());
    }

    #[test]
    fn effect_countdown_uses_accumulated_fixed_time_only() {
        let mut app = App::new();
        app.init_resource::<Time<Fixed>>().add_plugins(RosterPlugin);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_millis(500));
        app.world_mut().run_schedule(FixedUpdate);
        app.update();

        assert_eq!(
            app.world().resource::<ArenaRoster>().arenas[0].characters[0].effects[0]
                .remaining_seconds,
            39
        );

        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_millis(500));
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(
            app.world().resource::<ArenaRoster>().arenas[0].characters[0].effects[0]
                .remaining_seconds,
            38
        );

        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::ZERO);
        app.world_mut().run_schedule(FixedUpdate);
        app.update();
        assert_eq!(
            app.world().resource::<ArenaRoster>().arenas[0].characters[0].effects[0]
                .remaining_seconds,
            38
        );
    }

    #[test]
    fn arena_navigation_wraps_and_rejects_unknown_indices() {
        let mut roster = ArenaRoster::default();

        roster.cycle_arena(-1);
        assert_eq!(roster.active_arena, 8);
        roster.cycle_arena(1);
        assert_eq!(roster.active_arena, 0);
        roster.cycle_arena(i32::MIN);
        assert_eq!(roster.active_arena, 7);
        assert!(!roster.select_arena(ARENA_COUNT));
        assert_eq!(roster.active_arena, 7);
        assert!(roster.select_arena(0));
        assert_eq!(roster.active_arena, 0);
    }

    #[test]
    fn overflowing_characters_remain_selectable_and_cycle_with_the_roster() {
        let mut roster = ArenaRoster::default();
        assert_eq!(roster.current_arena().characters.len(), 41);
        assert!(roster.select_character(VISIBLE_ROSTER_SLOTS));
        assert_eq!(
            roster.selected_character().map(|character| character.id),
            Some(40)
        );

        roster.cycle_character(false);
        assert_eq!(roster.current_arena().selected, Some(0));
        roster.cycle_character(true);
        assert_eq!(roster.current_arena().selected, Some(40));
        assert!(!roster.select_character(41));
        assert_eq!(roster.current_arena().selected, Some(40));
    }

    #[test]
    fn selection_skips_dead_characters_and_handles_empty_or_all_dead_arenas() {
        let mut roster = ArenaRoster::default();
        assert!(roster.select_character(10));
        assert!(!roster.select_character(11));
        roster.cycle_character(false);
        assert_eq!(roster.current_arena().selected, Some(12));
        roster.cycle_character(true);
        assert_eq!(roster.current_arena().selected, Some(10));

        assert!(roster.select_arena(2));
        roster.cycle_character(true);
        assert!(roster.selected_character().is_none());
        assert!(!roster.select_character(0));

        assert!(roster.select_arena(0));
        for character in &mut roster.arenas[0].characters {
            character.hp = 0;
        }
        roster.cycle_character(false);
        assert!(roster.current_arena().selected.is_none());
        assert!(roster.selected_character().is_none());
    }

    #[test]
    fn switching_arenas_preserves_each_selection() {
        let mut roster = ArenaRoster::default();
        assert!(roster.select_character(5));
        assert!(roster.select_arena(1));
        assert!(roster.select_character(3));
        assert!(roster.select_arena(0));
        assert_eq!(roster.current_arena().selected, Some(5));
        assert!(roster.select_arena(1));
        assert_eq!(roster.current_arena().selected, Some(3));
    }

    #[test]
    fn debuffs_precede_buffs_and_each_group_orders_by_expiry() {
        let character = fixture_character(0);
        let names: Vec<_> = character
            .sorted_effects()
            .iter()
            .map(|effect| effect.name)
            .collect();

        assert_eq!(names, ["Frostbite", "Bleed", "Haste", "Regain"]);
        assert_eq!(
            character.class.abilities(),
            ["Block", "Bash", "Taunt", "Barrier"]
        );
    }
}
