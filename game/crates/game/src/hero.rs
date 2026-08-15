use bevy::prelude::*;

use crate::{
    abilities::{Abilities, Ability},
    tile::{self, GridPosition},
};

/// Radius of a one-tile-wide hero.
pub const RADIUS: f32 = tile::SIZE * 0.5;

/// Marks the player-controlled character.
#[derive(Component, Debug, Default, Clone, Copy)]
#[require(Abilities, GridPosition)]
pub struct Hero;

/// Marks a hero as part of the player's current selection.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Selected;

/// Marks a selected hero as currently able to receive player actions.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Active;

/// Describes the initially selected, player-controlled hero.
pub fn scene() -> impl Scene {
    let material = StandardMaterial {
        base_color: Color::oklch(0.732_032, 0.153_756, 240.894),
        unlit: true,
        ..default()
    };

    bsn! {
        Hero
        Abilities::new([Some(Ability::Bash), None, None, None])
        Selected
        Active
        Mesh3d(asset_value(Sphere::new(RADIUS)))
        MeshMaterial3d::<StandardMaterial>(asset_value(material))
        Transform::from_xyz(0.0, 0.0, RADIUS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hero_receives_four_empty_ability_slots_by_default() {
        let mut world = World::new();
        let hero = world.spawn(Hero).id();
        let abilities = world
            .get::<Abilities>(hero)
            .expect("invariant: Hero requires an Abilities component");

        assert_eq!(abilities.slots, [None; crate::abilities::SLOT_COUNT]);
    }
}
