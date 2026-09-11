const FIRST_NAMES: [&str; 100] = [
    "Arlen", "Asha", "Alden", "Alina", "Ansel", "Anya", "Aric", "Astrid", "Avery", "Azra",
    "Bastian", "Beatrix", "Bennett", "Briar", "Bryn", "Cadence", "Callan", "Cassia", "Cedric",
    "Celia", "Cian", "Clara", "Corin", "Dahlia", "Darian", "Delia", "Dorian", "Eira", "Elian",
    "Elise", "Ember", "Emrys", "Enid", "Evander", "Faye", "Felix", "Fenric", "Fiona", "Flora",
    "Galen", "Gemma", "Gideon", "Greta", "Hadrian", "Hana", "Hazel", "Hugo", "Idris", "Ilya",
    "Ines", "Iona", "Iris", "Isolde", "Jasper", "Jessa", "Jonas", "Juno", "Kael", "Kara", "Kieran",
    "Lara", "Leif", "Lenora", "Liora", "Lucan", "Lyra", "Maeve", "Magnus", "Mara", "Milo", "Mira",
    "Nadia", "Nessa", "Nico", "Nolan", "Nora", "Oren", "Orla", "Oscar", "Petra", "Quinn", "Rafe",
    "Rhea", "Ronan", "Rosalind", "Rowan", "Sabine", "Sable", "Sera", "Silas", "Soren", "Talia",
    "Tamsin", "Theo", "Tobin", "Una", "Vera", "Wren", "Yara", "Zev",
];

const LAST_NAMES: [&str; 100] = [
    "Ashford",
    "Amberfall",
    "Alderbrook",
    "Arden",
    "Ashvale",
    "Bellweather",
    "Blackthorn",
    "Brightwater",
    "Briarwood",
    "Brookstone",
    "Cinderhall",
    "Clearwell",
    "Cloudmere",
    "Copperfield",
    "Crowhurst",
    "Dawnbrook",
    "Deepwell",
    "Duskwood",
    "Duskmere",
    "Dunvale",
    "Eastwind",
    "Elderwood",
    "Emberfall",
    "Everhart",
    "Evenwood",
    "Fairbrook",
    "Faraday",
    "Fernwood",
    "Flintlock",
    "Foxglove",
    "Goldcrest",
    "Goodwin",
    "Graystone",
    "Greenbriar",
    "Groveward",
    "Halloway",
    "Hartwell",
    "Hawthorne",
    "Highwater",
    "Hollowbrook",
    "Ironbloom",
    "Ironcrest",
    "Iverstone",
    "Ivydale",
    "Ivoryvale",
    "Jadebrook",
    "Juniper",
    "Kingswell",
    "Kestrel",
    "Kindlewood",
    "Larkspur",
    "Lightfoot",
    "Longmere",
    "Lowell",
    "Lynden",
    "Meadowvale",
    "Merriwether",
    "Moonbrook",
    "Mossgrove",
    "Mistborne",
    "Nettleford",
    "Nightbloom",
    "Northwind",
    "Norwood",
    "Oakheart",
    "Oakley",
    "Oakhurst",
    "Oriel",
    "Pinebrook",
    "Pryce",
    "Quill",
    "Quarryborn",
    "Ravencrest",
    "Redfern",
    "Ridgewell",
    "Riverstone",
    "Rosewood",
    "Rowntree",
    "Silverleaf",
    "Snowden",
    "Springvale",
    "Starling",
    "Stoneward",
    "Summerfield",
    "Thistlewood",
    "Thornfield",
    "Timberfall",
    "Tidewell",
    "Underhill",
    "Umberfield",
    "Vale",
    "Verdant",
    "Westfall",
    "Whitethorn",
    "Wildmere",
    "Willowby",
    "Winterborne",
    "Woodward",
    "Yarrow",
    "Zephyr",
];

/// Generates a stable name from identity, so respawning never renames a character.
///
/// The low base-100 digit selects a first name. The high digit, offset by a
/// deterministic permutation of the low digit, selects a last name. Every pair
/// appears once per 10,000 IDs, while consecutive characters have varied surnames.
/// Names are display labels; IDs remain authoritative after the cycle repeats.
pub fn character_name(id: u32) -> String {
    let first = id % 100;
    let last = (id / 100).strict_add(first.strict_mul(37)) % 100;
    let first = usize::try_from(first).expect("invariant: name index is below 100");
    let last = usize::try_from(last).expect("invariant: name index is below 100");
    format!("{} {}", FIRST_NAMES[first], LAST_NAMES[last])
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::character_name;

    #[test]
    fn first_ten_thousand_identities_have_distinct_stable_names() {
        let names: HashSet<_> = (0..10_000).map(character_name).collect();

        assert_eq!(names.len(), 10_000);
        assert_eq!(character_name(0), "Arlen Ashford");
        assert_eq!(character_name(10_000), character_name(0));
        assert_eq!(character_name(u32::MAX), character_name(u32::MAX % 10_000));
    }
}
