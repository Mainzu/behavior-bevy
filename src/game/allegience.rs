use std::iter;

use super::*;

use bitflags::bitflags;
use enum_primitive_derive::Primitive;
use num_traits::{self, FromPrimitive, ToPrimitive};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Factions: u8 {
        const A = 1 << 0;
        const B = 1 << 1;
        const C = 1 << 2;
        const D = 1 << 3;
        const E = 1 << 4;
        const F = 1 << 5;
        const G = 1 << 6;
        const H = 1 << 7;

        const NONE = 0;
        const ALL = Self::A.bits() | Self::B.bits() | Self::C.bits() |
                    Self::D.bits() | Self::E.bits() | Self::F.bits() | Self::G.bits() | Self::H.bits();
    }
}

#[derive(Primitive, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    Neutral = 0b00,
    Allied = 0b01,
    Hostile = 0b10,
}

pub mod components {
    use super::*;

    use bevy::color::palettes::css::*;

    #[derive(Primitive, Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
    pub enum Faction {
        #[default]
        A = 0b00000001,
        B = 0b00000010,
        C = 0b00000100,
        D = 0b00001000,
        E = 0b00010000,
        F = 0b00100000,
        G = 0b01000000,
        H = 0b10000000,
    }
    impl Faction {
        pub fn iter_cycle() -> impl Iterator<Item = Faction> {
            let mut value: u8 = 1;

            iter::from_fn(move || {
                let result = Faction::from_u8(value);
                value = value.rotate_left(1);
                result
            })
        }
        pub fn iter_once() -> impl Iterator<Item = Faction> {
            let mut value: u8 = 1;

            iter::from_fn(move || {
                let result = Faction::from_u8(value);
                value <<= 1;
                result
            })
        }

        pub fn color(self) -> Color {
            match self {
                Faction::A => RED,
                Faction::B => ORANGE,
                Faction::C => YELLOW,
                Faction::D => GREEN,
                Faction::E => BLUE,
                Faction::F => INDIGO,
                Faction::G => PURPLE,
                Faction::H => PINK,
            }
            .into()
        }

        pub fn color_opt(faction: Option<Faction>) -> Color {
            faction.map(Faction::color).unwrap_or(GRAY.into())
        }
    }
}

pub mod resources {
    use super::*;

    use components::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
    pub struct FactionRelationships {
        relationships: u64,
    }

    impl FactionRelationships {
        pub fn new() -> Self {
            FactionRelationships { relationships: 0 }
        }

        pub fn with_default(default: Relationship) -> Self {
            let value = default as u64;
            let relationships = (0..28).fold(0u64, |acc, i| acc | (value << (i * 2)));
            FactionRelationships { relationships }
        }

        pub fn from_mapping(
            map: impl IntoIterator<Item = ((Faction, Faction), Relationship)>,
        ) -> Self {
            let mut fr = Self::new();
            for ((faction1, faction2), relationship) in map {
                fr.set_relationship(faction1, faction2, relationship);
            }
            fr
        }

        pub fn from_closure<F>(mut f: F) -> Self
        where
            F: FnMut(Faction, Faction) -> Relationship,
        {
            let mut fr = Self::new();
            for faction1 in Faction::iter_once() {
                for faction2 in Faction::iter_once() {
                    if (faction1 as u8) < (faction2 as u8) {
                        let relationship = f(faction1, faction2);
                        fr.set_relationship(faction1, faction2, relationship);
                    }
                }
            }
            fr
        }

        pub fn set_relationship(
            &mut self,
            faction1: Faction,
            faction2: Faction,
            relationship: Relationship,
        ) {
            let (index, shift) = self.get_index_and_shift(faction1, faction2);
            let value = relationship as u64;

            // Clear the existing bits
            self.relationships &= !(0b11 << shift);
            // Set the new relationship
            self.relationships |= value << shift;
        }

        pub fn get_relationship(&self, faction1: Faction, faction2: Faction) -> Relationship {
            let (index, shift) = self.get_index_and_shift(faction1, faction2);
            match (self.relationships >> shift) & 0b11 {
                0 => Relationship::Neutral,
                1 => Relationship::Allied,
                2 => Relationship::Hostile,
                _ => unreachable!(),
            }
        }

        fn get_index_and_shift(
            &self,
            mut faction1: Faction,
            mut faction2: Faction,
        ) -> (usize, usize) {
            // Ensure faction1 < faction2
            if (faction1 as u8) > (faction2 as u8) {
                std::mem::swap(&mut faction1, &mut faction2);
            }

            let f1 = faction1 as u8;
            let f2 = faction2 as u8;

            // Calculate the index in the lower triangular matrix
            let index = (f1 * 7 + f2 - 1) - (f1 * (f1 + 1) / 2);
            let shift = index * 2;

            (index as usize, shift as usize)
        }
    }
}

#[derive(Default)]
pub struct AllegiencePlugin {
    faction_relationships: resources::FactionRelationships,
}

impl Plugin for AllegiencePlugin {
    fn build(&self, app: &mut App) {
        use resources::*;

        app.insert_resource(self.faction_relationships);
    }
}

pub mod prelude {
    pub use super::components::*;
    pub use super::resources::*;

    pub use super::AllegiencePlugin;
}

#[cfg(test)]
mod tests {
    use super::*;

    use components::*;

    #[test]
    fn faction_cycle() {
        let mut factions = Faction::iter_cycle();

        assert_eq!(factions.next(), Some(Faction::A));
        assert_eq!(factions.next(), Some(Faction::B));
        assert_eq!(factions.next(), Some(Faction::C));
        assert_eq!(factions.next(), Some(Faction::D));
        assert_eq!(factions.next(), Some(Faction::E));
        assert_eq!(factions.next(), Some(Faction::F));
        assert_eq!(factions.next(), Some(Faction::G));
        assert_eq!(factions.next(), Some(Faction::H));

        assert_eq!(factions.next(), Some(Faction::A));
    }

    #[test]
    fn faction_once() {
        let mut factions = Faction::iter_once();

        assert_eq!(factions.next(), Some(Faction::A));
        assert_eq!(factions.next(), Some(Faction::B));
        assert_eq!(factions.next(), Some(Faction::C));
        assert_eq!(factions.next(), Some(Faction::D));
        assert_eq!(factions.next(), Some(Faction::E));
        assert_eq!(factions.next(), Some(Faction::F));
        assert_eq!(factions.next(), Some(Faction::G));
        assert_eq!(factions.next(), Some(Faction::H));

        assert_eq!(factions.next(), None);
    }
}
