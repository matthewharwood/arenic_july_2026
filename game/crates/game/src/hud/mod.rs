//! Native game HUD: presentation and device adapters for arena/hero selection.
mod feedback;
mod view;

use bevy::{
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    window::PrimaryWindow,
};

use crate::movement::MoveHero;
use crate::roster::{ArenaRoster, RosterPlugin};
use crate::theme as tokens;
use feedback::{CharacterDelta, FeedbackAnchor, FeedbackPlugin, FeedbackSet};

pub struct HudPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HudSelectionSet;

#[derive(Component)]
struct CharacterCard;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RosterPlugin)
            .init_resource::<HudPresentation>()
            .init_resource::<StatDisplay>()
            .init_resource::<InputFocus>()
            .add_message::<HudAction>()
            .add_plugins(FeedbackPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    (keyboard_actions, pointer_actions).chain(),
                    apply_actions,
                    sync_stat_targets.run_if(resource_changed::<ArenaRoster>),
                    refresh.run_if(hud_changed),
                )
                    .chain()
                    .before(FeedbackSet::Present),
            )
            .add_systems(
                Update,
                (
                    fit_hud,
                    sync_selected_hero
                        .after(apply_actions)
                        .in_set(HudSelectionSet)
                        .run_if(resource_changed::<ArenaRoster>),
                    animate_bars.run_if(stats_are_animating).after(refresh),
                    animate_buttons.after(refresh).run_if(buttons_are_animating),
                ),
            )
            .add_systems(
                PostUpdate,
                position_feedback_anchor.after(bevy::ui::UiSystems::Layout),
            );
    }
}

fn position_feedback_anchor(
    cards: Query<(&ComputedNode, &UiGlobalTransform), With<CharacterCard>>,
    mut anchors: Query<&mut Node, With<FeedbackAnchor>>,
) {
    let Ok((computed, transform)) = cards.single() else {
        return;
    };
    let Ok(mut anchor) = anchors.single_mut() else {
        return;
    };
    let corner = (transform.translation + Vec2::new(computed.size().x, -computed.size().y) * 0.5)
        * computed.inverse_scale_factor();
    let left = px(corner.x - 40.0);
    let top = px(corner.y + 28.0);
    if anchor.left != left || anchor.top != top {
        anchor.left = left;
        anchor.top = top;
        anchor.bottom = Val::Auto;
    }
}

fn sync_selected_hero(
    mut commands: Commands,
    roster: Res<ArenaRoster>,
    mut previous: Local<Option<u32>>,
    mut pending_moves: ResMut<Messages<MoveHero>>,
    mut heroes: Query<(Entity, &mut Visibility, &mut Transform), With<crate::hero::Hero>>,
) {
    if !roster.is_changed() {
        return;
    }
    let selected = roster.selected_character().map(|character| character.id);
    if selected == *previous {
        return;
    }
    *previous = selected;
    // More than one rendered frame may precede a fixed step. Discard the prior
    // character's intent before the ordered hardware adapter reads fresh input.
    pending_moves.clear();
    // The current arena scene uses one hero preview; the roster owns selection.
    for (entity, mut visibility, mut transform) in &mut heroes {
        if selected.is_some() {
            *visibility = Visibility::Visible;
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            commands.entity(entity).insert(crate::hero::Active);
        } else {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<crate::hero::Active>();
        }
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component, Message, Debug, Clone, Copy, PartialEq, Eq)]
enum HudAction {
    PreviousHero,
    NextHero,
    SelectHero(usize),
    PreviousArena,
    NextArena,
    SelectArena(usize),
    Ability(usize),
    Record,
    ToggleOverflow,
    ToggleHelp,
    PreviewFeedback,
}

#[derive(Resource, Default)]
struct HudPresentation {
    overflow_open: bool,
    help_open: bool,
}

#[derive(Component)]
struct ButtonPulse {
    remaining: f32,
    rest: Color,
}

#[derive(Component, Clone, Copy)]
enum StatBar {
    Experience,
    Health,
}

#[derive(Resource, Default)]
struct StatDisplay {
    character_id: Option<u32>,
    xp: f32,
    hp: f32,
    target_xp: f32,
    target_hp: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
    // This layer survives card rebuilds, allowing stat feedback to finish.
    commands.spawn((
        FeedbackAnchor,
        Node {
            position_type: PositionType::Absolute,
            left: px(268),
            bottom: px(134),
            width: px(270),
            height: px(1),
            ..default()
        },
        GlobalZIndex(30),
    ));
}

fn fit_hud(windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    if let Ok(window) = windows.single() {
        // A single proportional layout keeps all four slots and nine arenas on screen.
        let next = (window.width() / 1280.0).min(1.5);
        if next > 0.0 && (scale.0 - next).abs() > 0.001 {
            scale.0 = next;
        }
    }
}

fn keyboard_actions(keys: Res<ButtonInput<KeyCode>>, mut actions: MessageWriter<HudAction>) {
    // Browser and operating-system shortcuts must not also trigger game actions.
    if keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]) {
        return;
    }
    // At most one keyboard intent per frame, with navigation taking priority.
    let action = if keys.just_pressed(KeyCode::BracketLeft) {
        Some(HudAction::PreviousArena)
    } else if keys.just_pressed(KeyCode::BracketRight) {
        Some(HudAction::NextArena)
    } else if keys.just_pressed(KeyCode::Tab) {
        Some(
            if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                HudAction::PreviousHero
            } else {
                HudAction::NextHero
            },
        )
    } else if keys.just_pressed(KeyCode::KeyH) {
        Some(HudAction::ToggleHelp)
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(HudAction::Record)
    } else {
        [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
        ]
        .into_iter()
        .position(|key| keys.just_pressed(key))
        .map(HudAction::Ability)
    };
    if let Some(action) = action {
        actions.write(action);
    }
}

fn pointer_actions(
    buttons: Query<(Entity, &Interaction, &HudAction), ChangedButton>,
    mut focus: ResMut<InputFocus>,
    mut actions: MessageWriter<HudAction>,
) {
    // Buttons do not overlap; only the clicked button produces an intent.
    for (entity, interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            focus.set(entity, FocusCause::Pressed);
            actions.write(*action);
            break;
        }
    }
}

type ChangedButton = (Changed<Interaction>, With<Button>);

fn apply_actions(
    mut actions: MessageReader<HudAction>,
    mut roster: ResMut<ArenaRoster>,
    mut presentation: ResMut<HudPresentation>,
    mut pulses: Query<(&HudAction, &mut ButtonPulse)>,
    mut feedback: MessageWriter<CharacterDelta>,
) {
    // Keyboard is written before pointer input; consume at most one action per frame.
    let action = actions.read().next().copied();
    actions.clear();
    let Some(action) = action else {
        return;
    };
    match action {
        HudAction::PreviousHero => {
            roster.cycle_character(true);
        }
        HudAction::NextHero => {
            roster.cycle_character(false);
        }
        HudAction::SelectHero(index) => {
            roster.select_character(index);
        }
        HudAction::PreviousArena => {
            roster.cycle_arena(-1);
            presentation.overflow_open = false;
        }
        HudAction::NextArena => {
            roster.cycle_arena(1);
            presentation.overflow_open = false;
        }
        HudAction::SelectArena(index) => {
            roster.select_arena(index);
            presentation.overflow_open = false;
        }
        HudAction::ToggleOverflow => {
            presentation.overflow_open = !presentation.overflow_open;
        }
        HudAction::ToggleHelp => {
            presentation.help_open = !presentation.help_open;
        }
        HudAction::PreviewFeedback => {
            if let Some(character) = roster.selected_character() {
                feedback.write(CharacterDelta {
                    character_id: character.id,
                    experience: 9,
                    hp: -9,
                });
            }
        }
        HudAction::Ability(_) | HudAction::Record => {
            // Ability execution and recording are intentionally not simulated by the HUD.
            if roster.selected_character().is_some() || action == HudAction::Record {
                for (button_action, mut pulse) in &mut pulses {
                    if *button_action == action {
                        pulse.remaining = 0.18;
                    }
                }
            }
        }
    }
}

fn sync_stat_targets(roster: Res<ArenaRoster>, mut stats: ResMut<StatDisplay>) {
    let character = roster.selected_character();
    let id = character.map(|character| character.id);
    let xp = character.map_or(0.0, |character| {
        fraction(character.experience, character.next_level_experience)
    });
    let hp = character.map_or(0.0, |character| fraction(character.hp, character.max_hp));
    if stats.character_id != id {
        stats.xp = xp;
        stats.hp = hp;
        stats.character_id = id;
    }
    stats.target_xp = xp;
    stats.target_hp = hp;
}

fn fraction(value: u32, maximum: u32) -> f32 {
    if maximum == 0 {
        0.0
    } else {
        (value as f32 / maximum as f32).clamp(0.0, 1.0)
    }
}

fn refresh(
    mut commands: Commands,
    roots: Query<Entity, With<HudRoot>>,
    roster: Res<ArenaRoster>,
    presentation: Res<HudPresentation>,
    stats: Res<StatDisplay>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    view::spawn(&mut commands, &roster, &presentation, &stats);
}

fn hud_changed(roster: Res<ArenaRoster>, presentation: Res<HudPresentation>) -> bool {
    roster.is_changed() || presentation.is_changed()
}

fn buttons_are_animating(buttons: Query<&ButtonPulse>) -> bool {
    buttons.iter().any(|pulse| pulse.remaining > 0.0)
}

fn stats_are_animating(stats: Res<StatDisplay>) -> bool {
    (stats.xp - stats.target_xp).abs() > 0.0001 || (stats.hp - stats.target_hp).abs() > 0.0001
}

fn animate_bars(
    time: Res<Time>,
    mut stats: ResMut<StatDisplay>,
    mut bars: Query<(&StatBar, &mut Node)>,
) {
    let blend = (time.delta_secs() * 9.0).min(1.0);
    stats.xp += (stats.target_xp - stats.xp) * blend;
    stats.hp += (stats.target_hp - stats.hp) * blend;
    for (kind, mut node) in &mut bars {
        node.width = percent(
            match kind {
                StatBar::Experience => stats.xp,
                StatBar::Health => stats.hp,
            } * 100.0,
        );
    }
}

fn animate_buttons(time: Res<Time>, mut buttons: Query<(&mut ButtonPulse, &mut BackgroundColor)>) {
    for (mut pulse, mut background) in &mut buttons {
        if pulse.remaining > 0.0 {
            pulse.remaining = (pulse.remaining - time.delta_secs()).max(0.0);
            background.0 = if pulse.remaining > 0.0 {
                tokens::TRACK
            } else {
                pulse.rest
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hero::{Active, Hero, Selected},
        movement::HeroMovementPlugin,
        tile,
    };

    #[test]
    fn selection_change_discards_old_movement_before_the_next_fixed_step() {
        for (fresh_input, expected_position) in [
            (None, Vec3::ZERO),
            (Some(GamepadButton::DPadUp), Vec3::new(0.0, tile::SIZE, 0.0)),
        ] {
            let mut app = App::new();
            app.init_resource::<ArenaRoster>()
                .add_plugins(HeroMovementPlugin)
                .add_systems(Update, sync_selected_hero.in_set(HudSelectionSet));
            let hero = app
                .world_mut()
                .spawn((
                    Hero,
                    Selected,
                    Active,
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            let gamepad = app.world_mut().spawn(Gamepad::default()).id();
            app.world_mut()
                .get_mut::<Gamepad>(gamepad)
                .expect("invariant: the test gamepad entity has a Gamepad component")
                .digital_mut()
                .press(GamepadButton::DPadRight);
            app.update();
            assert_eq!(app.world().resource::<Messages<MoveHero>>().len(), 1);

            assert!(
                app.world_mut()
                    .resource_mut::<ArenaRoster>()
                    .select_character(1)
            );
            {
                let mut controller = app
                    .world_mut()
                    .get_mut::<Gamepad>(gamepad)
                    .expect("invariant: the test gamepad entity has a Gamepad component");
                controller.digital_mut().reset_all();
                if let Some(button) = fresh_input {
                    controller.digital_mut().press(button);
                }
            }
            // Two Update schedules precede the first fixed step, as they can
            // when rendering is faster than the simulation tick rate.
            app.update();
            app.world_mut().run_schedule(FixedUpdate);

            assert_eq!(
                app.world()
                    .get::<Transform>(hero)
                    .expect("invariant: the hero preview has a Transform")
                    .translation,
                expected_position
            );
        }
    }

    #[test]
    fn stat_fractions_handle_empty_and_overfilled_bars() {
        assert_eq!(fraction(9, 0), 0.0);
        assert_eq!(fraction(120, 100), 1.0);
        assert_eq!(fraction(25, 100), 0.25);
    }

    #[test]
    fn browser_shortcuts_do_not_also_emit_game_actions() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .add_message::<HudAction>()
            .add_systems(Update, keyboard_actions);
        for (modifier, key) in [
            (KeyCode::ControlLeft, KeyCode::Tab),
            (KeyCode::SuperLeft, KeyCode::Digit1),
            (KeyCode::AltRight, KeyCode::BracketLeft),
        ] {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(modifier);
            keys.press(key);
            app.update();
            assert!(app.world().resource::<Messages<HudAction>>().is_empty());
        }
    }

    #[test]
    fn bracket_and_shift_tab_intents_change_the_expected_selection() {
        let mut app = App::new();
        app.init_resource::<ArenaRoster>()
            .init_resource::<HudPresentation>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<HudAction>()
            .add_message::<CharacterDelta>()
            .add_systems(Update, (keyboard_actions, apply_actions).chain());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::BracketLeft);
        app.update();
        assert_eq!(app.world().resource::<ArenaRoster>().active_arena, 8);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ArenaRoster>()
            .select_arena(0);
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::Tab);
        app.update();
        assert_eq!(
            app.world()
                .resource::<ArenaRoster>()
                .current_arena()
                .selected,
            Some(40)
        );
    }
}
