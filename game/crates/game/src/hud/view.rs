use bevy::prelude::*;

use super::{
    ButtonPulse, CharacterCard, HudAction, HudPresentation, HudRoot, StatBar, StatDisplay,
    tokens::*,
};
use crate::roster::{ArenaRoster, Character, VISIBLE_ROSTER_SLOTS};

pub(super) fn spawn(
    commands: &mut Commands,
    roster: &ArenaRoster,
    presentation: &HudPresentation,
    stats: &StatDisplay,
) {
    let root = commands
        .spawn((
            HudRoot,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .id();
    header(commands, root, roster);
    let dock = panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            width: percent(100),
            height: px(194),
            padding: UiRect::axes(px(24), px(16)),
            border: UiRect::top(px(1)),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::px(218.0),
                GridTrack::px(272.0),
                GridTrack::flex(1.0),
                GridTrack::px(78.0),
                GridTrack::px(142.0),
            ],
            align_items: AlignItems::FlexEnd,
            column_gap: px(18),
            ..default()
        },
        SURFACE,
    );
    commands.entity(dock).insert(BorderColor::all(TRACK));
    roster_panel(commands, dock, roster, presentation);
    character_card(commands, dock, roster.selected_character(), stats);
    abilities(commands, dock, roster.selected_character());
    record_button(commands, dock);
    arena_map(commands, dock, roster);
    if presentation.help_open {
        help(commands, root, roster.selected_character().is_some());
    }
}

fn header(commands: &mut Commands, root: Entity, roster: &ArenaRoster) {
    let header = panel(
        commands,
        root,
        Node {
            width: percent(100),
            height: px(46),
            padding: UiRect::axes(px(24), px(0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(px(1)),
            ..default()
        },
        SURFACE,
    );
    commands.entity(header).insert(BorderColor::all(TRACK));
    let title = row(commands, header, 10.0);
    label(commands, title, "ARENIC", TITLE, INK);
    label(commands, title, "/", BODY, BORDER);
    label(
        commands,
        title,
        roster.current_arena().name.to_uppercase(),
        BODY,
        MUTED,
    );
    let right = row(commands, header, 16.0);
    label(
        commands,
        right,
        format!("ARENA {:02} / 09", roster.active_arena.strict_add(1)),
        SMALL,
        MUTED,
    );
    key_button(commands, right, "Controls  H", HudAction::ToggleHelp, 94.0);
}

/// The selected overflow character replaces the last visible slot; no identity is lost.
fn visible_indices(roster: &ArenaRoster) -> [Option<usize>; VISIBLE_ROSTER_SLOTS] {
    let arena = roster.current_arena();
    let mut visible =
        std::array::from_fn(|index| (index < arena.characters.len()).then_some(index));
    if let Some(selected) = arena.selected
        && selected >= VISIBLE_ROSTER_SLOTS
    {
        visible[VISIBLE_ROSTER_SLOTS.strict_sub(1)] = Some(selected);
    }
    visible
}

fn roster_panel(
    commands: &mut Commands,
    dock: Entity,
    roster: &ArenaRoster,
    presentation: &HudPresentation,
) {
    let arena = roster.current_arena();
    let area = column(commands, dock, 218.0, 7.0);
    let navigation = row(commands, area, 5.0);
    key_button(
        commands,
        navigation,
        "< Shift Tab",
        HudAction::PreviousHero,
        108.0,
    );
    key_button(commands, navigation, "Tab >", HudAction::NextHero, 62.0);
    let caption = row(commands, area, 10.0);
    label(commands, caption, "CHARACTERS", SMALL, MUTED);
    label(
        commands,
        caption,
        format!("{} / 40", arena.characters.len()),
        SMALL,
        INK,
    );
    let grid = panel(
        commands,
        area,
        Node {
            width: px(214),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(10, 18.0),
            grid_template_rows: RepeatedGridTrack::px(4, 18.0),
            column_gap: px(3),
            row_gap: px(3),
            ..default()
        },
        SURFACE,
    );
    let visible = visible_indices(roster);
    for index in visible {
        avatar(
            commands,
            grid,
            index.and_then(|index| arena.characters.get(index)),
            index,
            index.is_some() && arena.selected == index,
        );
    }
    let overflow = arena.characters.len().saturating_sub(VISIBLE_ROSTER_SLOTS);
    if overflow > 0 {
        key_button(
            commands,
            area,
            &format!(
                "+{overflow} reserve  {}",
                if presentation.overflow_open { "-" } else { "+" }
            ),
            HudAction::ToggleOverflow,
            125.0,
        );
        if presentation.overflow_open {
            let tray = panel(
                commands,
                area,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: percent(100),
                    left: px(0),
                    margin: UiRect::bottom(px(8)),
                    width: px(236),
                    padding: UiRect::all(px(12)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                SURFACE,
            );
            commands
                .entity(tray)
                .insert((BorderColor::all(OVERLAY), ZIndex(5)));
            label(commands, tray, "RESERVE CHARACTERS", SMALL, MUTED);
            for (index, character) in arena
                .characters
                .iter()
                .enumerate()
                .filter(|(index, _)| !visible.contains(&Some(*index)))
            {
                let item = panel(
                    commands,
                    tray,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        ..default()
                    },
                    SURFACE,
                );
                avatar(commands, item, Some(character), Some(index), false);
                let button = button(
                    commands,
                    item,
                    HudAction::SelectHero(index),
                    Node {
                        padding: UiRect::all(px(3)),
                        ..default()
                    },
                    SURFACE,
                    SURFACE,
                );
                label(commands, button, &character.name, SMALL, INK);
            }
            label(
                commands,
                tray,
                "Selected heroes stay in view.",
                SMALL,
                MUTED,
            );
        }
    } else {
        label(commands, area, "X empty   /   skull fallen", SMALL, MUTED);
    }
}

fn avatar(
    commands: &mut Commands,
    parent: Entity,
    character: Option<&Character>,
    index: Option<usize>,
    selected: bool,
) {
    let node = Node {
        width: px(18),
        height: px(18),
        flex_shrink: 0.0,
        border: UiRect::all(px(if character.is_some() { 1.0 } else { 0.0 })),
        border_radius: BorderRadius::MAX,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    let background = if selected { SELECTED } else { SURFACE };
    let border = if selected { SELECTED } else { INK };
    let entity = panel(commands, parent, node, background);
    commands.entity(entity).insert(BorderColor::all(border));
    match character {
        None => {
            label(commands, entity, "X", BODY, BORDER);
        }
        Some(character) if character.hp == 0 => {
            skull(commands, entity);
        }
        Some(character) => {
            if let Some(index) = index {
                commands
                    .entity(entity)
                    .insert((Button, HudAction::SelectHero(index)));
            }
            label(
                commands,
                entity,
                character.class.initial(),
                SMALL,
                if selected { SURFACE } else { INK },
            );
        }
    }
}

// A vector-like skull keeps the fallen state visible without relying on font glyphs.
fn skull(commands: &mut Commands, parent: Entity) {
    let head = panel(
        commands,
        parent,
        Node {
            width: px(10),
            height: px(9),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        INK,
    );
    for left in [2.0, 6.0] {
        panel(
            commands,
            head,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(3),
                width: px(2),
                height: px(2),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            SURFACE,
        );
    }
    panel(
        commands,
        head,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(-2),
            left: px(2),
            width: px(6),
            height: px(3),
            ..default()
        },
        INK,
    );
}

fn character_card(
    commands: &mut Commands,
    dock: Entity,
    character: Option<&Character>,
    stats: &StatDisplay,
) {
    let card = column(commands, dock, 272.0, 5.0);
    commands.entity(card).insert(CharacterCard);
    let Some(character) = character else {
        label(commands, card, "No character selected", TITLE, INK);
        label(commands, card, "This arena is resting.", BODY, MUTED);
        label(
            commands,
            card,
            "Use [ ] to find your next hero.",
            SMALL,
            MUTED,
        );
        return;
    };
    label(
        commands,
        card,
        format!("{} | {}", character.class.label(), character.name),
        BODY,
        INK,
    );
    let experience = row(commands, card, 12.0);
    label(
        commands,
        experience,
        format!("LVL {:02}", character.level),
        SMALL,
        INK,
    );
    label(
        commands,
        experience,
        format!(
            "XP {} / {}",
            character.experience, character.next_level_experience
        ),
        SMALL,
        MUTED,
    );
    stat_bar(commands, card, stats.xp, XP, StatBar::Experience);
    label(
        commands,
        card,
        format!("HP  {} / {}", character.hp, character.max_hp),
        SMALL,
        INK,
    );
    stat_bar(commands, card, stats.hp, HP, StatBar::Health);
    let effects = panel(
        commands,
        card,
        Node {
            width: percent(100),
            min_height: px(35),
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(9),
            row_gap: px(4),
            margin: UiRect::top(px(3)),
            ..default()
        },
        SURFACE,
    );
    if character.effects.is_empty() {
        label(commands, effects, "No active effects", SMALL, MUTED);
    }
    for effect in character.sorted_effects() {
        let tag = row(commands, effects, 3.0);
        let name = if effect.stacks > 1 {
            format!("{} x{}", effect.name, effect.stacks)
        } else {
            effect.name.to_string()
        };
        label(commands, tag, name, SMALL, INK);
        label(
            commands,
            tag,
            format!("({}s)", effect.remaining_seconds),
            SMALL,
            if effect.beneficial {
                POSITIVE
            } else {
                NEGATIVE
            },
        );
    }
}

fn stat_bar(commands: &mut Commands, card: Entity, value: f32, color: Color, kind: StatBar) {
    let track = panel(
        commands,
        card,
        Node {
            width: percent(100),
            height: px(4),
            ..default()
        },
        TRACK,
    );
    let fill = panel(
        commands,
        track,
        Node {
            width: percent(value * 100.0),
            height: percent(100),
            ..default()
        },
        color,
    );
    commands.entity(fill).insert(kind);
}

fn abilities(commands: &mut Commands, dock: Entity, character: Option<&Character>) {
    let area = column(commands, dock, 292.0, 9.0);
    commands.entity(area).insert(Node {
        width: px(292),
        height: px(112),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        justify_self: JustifySelf::Center,
        ..default()
    });
    label(commands, area, "ABILITIES", SMALL, MUTED);
    let slots = row(commands, area, 8.0);
    commands.entity(slots).insert(Node {
        height: px(70),
        min_height: px(70),
        width: px(292),
        column_gap: px(8),
        ..default()
    });
    let names = character.map(|character| character.class.abilities());
    for index in 0..4 {
        let slot = button(
            commands,
            slots,
            HudAction::Ability(index),
            Node {
                width: px(67),
                height: px(70),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(12)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            SURFACE,
            if character.is_some() { INK } else { BORDER },
        );
        label(
            commands,
            slot,
            names.map_or("--", |names| names[index]),
            SMALL,
            if character.is_some() { INK } else { MUTED },
        );
        corner_label(commands, slot, index.strict_add(1).to_string(), SELECTED);
    }
    label(
        commands,
        area,
        "Always mapped to 1 / 2 / 3 / 4",
        SMALL,
        MUTED,
    );
}

fn record_button(commands: &mut Commands, dock: Entity) {
    let area = column(commands, dock, 78.0, 9.0);
    label(commands, area, "RECORDING", SMALL, MUTED);
    let record = button(
        commands,
        area,
        HudAction::Record,
        Node {
            width: px(76),
            height: px(70),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        SURFACE,
        NEGATIVE,
    );
    label(commands, record, "Record", BODY, NEGATIVE);
    corner_label(commands, record, "R", NEGATIVE);
    label(commands, area, "Coming later", SMALL, MUTED);
}

fn arena_map(commands: &mut Commands, dock: Entity, roster: &ArenaRoster) {
    let area = column(commands, dock, 142.0, 8.0);
    label(commands, area, "Raid: Normal", BODY, INK);
    let navigation = row(commands, area, 8.0);
    let grid = panel(
        commands,
        navigation,
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(3, 28.0),
            grid_template_rows: RepeatedGridTrack::px(3, 20.0),
            column_gap: px(4),
            row_gap: px(4),
            ..default()
        },
        SURFACE,
    );
    for (index, arena) in roster.arenas.iter().enumerate() {
        let active = index == roster.active_arena;
        let cell = button(
            commands,
            grid,
            HudAction::SelectArena(index),
            Node {
                width: px(28),
                height: px(20),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            if active { INK } else { SURFACE },
            if active { INK } else { BORDER },
        );
        label(
            commands,
            cell,
            if arena.characters.is_empty() && !active {
                "X".into()
            } else {
                format!("{:02}", index.strict_add(1))
            },
            SMALL,
            if active { SURFACE } else { MUTED },
        );
        if arena.alert {
            let dot = panel(
                commands,
                cell,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(-4),
                    right: px(-4),
                    width: px(9),
                    height: px(9),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                ALERT,
            );
            commands.entity(dot).insert(BorderColor::all(SURFACE));
        }
    }
    let paging = column(commands, navigation, 35.0, 8.0);
    key_button(commands, paging, "[ <", HudAction::PreviousArena, 35.0);
    key_button(commands, paging, "] >", HudAction::NextArena, 35.0);
    label(commands, area, "[ ] Change arena", SMALL, MUTED);
}

fn help(commands: &mut Commands, root: Entity, selected: bool) {
    let help = panel(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            top: px(62),
            right: px(24),
            width: px(334),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            row_gap: px(13),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        SURFACE,
    );
    commands
        .entity(help)
        .insert((BorderColor::all(OVERLAY), ZIndex(20)));
    label(commands, help, "FIND YOUR WAY", TITLE, INK);
    for (key, instruction) in [
        ("Tab / Shift Tab", "Next / previous living hero"),
        ("[ / ]", "Previous / next arena"),
        ("1 / 2 / 3 / 4", "Fixed ability slots"),
        ("R", "Recording shortcut (reserved)"),
        ("D-pad / left stick", "Move / click to change camera"),
    ] {
        let item = column(commands, help, 285.0, 3.0);
        label(commands, item, key, BODY, SELECTED);
        label(commands, item, instruction, SMALL, MUTED);
    }
    label(
        commands,
        help,
        "Blue: selected / HP    Green: XP / gains\nRed: damage / harmful effects / alerts",
        SMALL,
        MUTED,
    );
    if selected {
        key_button(
            commands,
            help,
            "Preview +9 XP / -9 HP",
            HudAction::PreviewFeedback,
            250.0,
        );
    }
    key_button(
        commands,
        help,
        "Close controls  H",
        HudAction::ToggleHelp,
        180.0,
    );
}

fn panel(commands: &mut Commands, parent: Entity, node: Node, color: Color) -> Entity {
    commands
        .spawn((node, BackgroundColor(color), ChildOf(parent)))
        .id()
}

fn row(commands: &mut Commands, parent: Entity, gap: f32) -> Entity {
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: px(gap),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

fn column(commands: &mut Commands, parent: Entity, width: f32, gap: f32) -> Entity {
    commands
        .spawn((
            Node {
                width: px(width),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(gap),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

fn label(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Text::new(text),
            TextFont::from_font_size(size),
            TextColor(color),
            ChildOf(parent),
        ))
        .id()
}

fn button(
    commands: &mut Commands,
    parent: Entity,
    action: HudAction,
    node: Node,
    background: Color,
    border: Color,
) -> Entity {
    commands
        .spawn((
            Button,
            action,
            node,
            BackgroundColor(background),
            BorderColor::all(border),
            ButtonPulse {
                remaining: 0.0,
                rest: background,
            },
            ChildOf(parent),
        ))
        .id()
}

fn key_button(
    commands: &mut Commands,
    parent: Entity,
    text: &str,
    action: HudAction,
    width: f32,
) -> Entity {
    let entity = button(
        commands,
        parent,
        action,
        Node {
            width: px(width),
            height: px(23),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        SURFACE,
        BORDER,
    );
    label(commands, entity, text, SMALL, SELECTED);
    entity
}

fn corner_label(commands: &mut Commands, parent: Entity, text: impl Into<String>, color: Color) {
    let entity = label(commands, parent, text, BODY, color);
    commands.entity(entity).insert(Node {
        position_type: PositionType::Absolute,
        bottom: px(4),
        right: px(6),
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_selection_stays_visible_without_duplicate_slots() {
        let mut roster = ArenaRoster::default();
        roster.select_character(40);
        let visible = visible_indices(&roster);
        assert_eq!(visible.len(), 40);
        assert_eq!(visible[39], Some(40));
        assert!(!visible.contains(&Some(39)));
        let unique: std::collections::HashSet<_> = visible.into_iter().flatten().collect();
        assert_eq!(unique.len(), 40);
    }

    #[test]
    fn empty_arena_keeps_all_four_ability_slots_and_nine_map_cells() {
        let mut app = App::new();
        app.init_resource::<ArenaRoster>()
            .init_resource::<HudPresentation>()
            .init_resource::<StatDisplay>();
        app.world_mut()
            .resource_mut::<ArenaRoster>()
            .select_arena(2);
        app.add_systems(Startup, super::super::refresh);
        app.update();
        let mut query = app.world_mut().query::<&HudAction>();
        let actions: Vec<_> = query.iter(app.world()).copied().collect();
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(action, HudAction::Ability(_)))
                .count(),
            4
        );
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(action, HudAction::SelectArena(_)))
                .count(),
            9
        );
    }
}
