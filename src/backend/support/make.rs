// use super::super::*;
// use enum_map::EnumMap;
// use rand::prelude::*;

// pub fn level(state: &mut State, map: &str) {
//     let mut loc = Point::origin();
//     for ch in map.chars() {
//         // TODO: If we keep these level files we may want to add a symbol mapping section
//         // so that letters can do things like refer to different uniques.
//         let _ = match ch {
//             ' ' => state.create_terrain(&loc, ObjectName::Dirt),
//             '#' => state.create_terrain(&loc, ObjectName::StoneWall),
//             'M' => state.create_terrain(&loc, ObjectName::MetalWall),
//             '+' => state.create_terrain(&loc, ObjectName::ClosedDoor),
//             '~' => state.create_terrain(&loc, ObjectName::ShallowWater),
//             'V' => state.create_terrain(&loc, ObjectName::Vitr),
//             'T' => state.create_terrain(&loc, ObjectName::Tree),
//             'W' => state.create_terrain(&loc, ObjectName::DeepWater),
//             'P' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Player)
//             }
//             'D' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Doorman)
//             }
//             'I' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Icarium)
//             }
//             'g' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Guard)
//             }
//             'o' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Spectator)
//             }
//             'R' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_char(&loc, ObjectName::Rhulad)
//             }
//             's' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_item(&loc, weak_sword(state))
//             }
//             'p' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_item(&loc, ObjectName::PickAxe)
//             }
//             'S' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_item(&loc, ObjectName::MightySword)
//             }
//             'a' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_item(&loc, ObjectName::LesserArmorySign)
//             }
//             'b' => {
//                 state.create_terrain(&loc, ObjectName::Dirt);
//                 state.create_item(&loc, ObjectName::GreaterArmorySign)
//             }
//             '\n' => Oid(0),
//             _ => {
//                 state.messages.push(Message {
//                     topic: Topic::Error,
//                     text: format!("Ignoring map char '{ch}'"),
//                 });
//                 state.create_terrain(&loc, ObjectName::Dirt)
//             }
//         };
//         if ch == '\n' {
//             loc = Point::new(0, loc.y + 1);
//         } else {
//             loc = Point::new(loc.x + 1, loc.y);
//         }
//     }
//     add_extras(state);
// }

// fn add_extras(state: &mut State) {
//     add_extra(state, ObjectName::LeatherHat);
//     add_extra(state, ObjectName::LeatherChest);
//     add_extra(state, ObjectName::LeatherGloves);
//     add_extra(state, ObjectName::LeatherLegs);
//     add_extra(state, ObjectName::LeatherSandals);
// }

// fn add_extra(state: &mut State, obj: Object) {
//     let mut count = 0;
//     {
//         let rng = &mut *state.rng();
//         while rng.gen_bool(0.5) {
//             count += 1;
//         }
//     }

//     for _ in 0..count {
//         for _ in 0..5 {
//             // we'll try 5x to add count instances of obj
//             let loc = state.level.random_loc(&state.rng);
//             let has_char = state.level.get(&loc, CHARACTER_ID).is_some();
//             let terrain = state.level.get_bottom(&loc).1;
//             if Terrain::Ground == terrain.terrain_value().unwrap() {
//                 if !has_char {
//                     state.create_item(&loc, obj.clone());
//                     break;
//                 }
//             }
//         }
//     }
// }

// fn weak_sword(state: &State) -> Object {
//     let swords = vec![
//         ObjectName::LongSword,
//         ObjectName::Broadsword,
//         ObjectName::LongKnife,
//         ObjectName::Dagger,
//     ];
//     let sword = swords.iter().choose(&mut *state.rng()).unwrap();
//     new_obj(*sword)
// }

// fn broken_name(name: ObjectName) -> &'static str {
//     use ObjectName::*;
//     match name {
//         BerokeSoftVoice => "Beroke Soft Voice",
//         HaladRackBearer => "Halad Rack Bearer",
//         ImrothTheCruel => "Imroth the Cruel",
//         KahlbTheSilentHunter => "Kahlb the SilentHunter",
//         SiballeTheUnfound => "Siballe the Unfound",
//         ThenikTheShattered => "Thenik the Shattered",
//         UrugalTheWoven => "Urugal the Woven",
//         _ => panic!("expected one of the broken, not {name:?}"),
//     }
// }
