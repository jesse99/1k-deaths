use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

enum Tag {
    S(&'static str),               // Tag name
    P(&'static str, &'static str), // Tag and type names
}

#[rustfmt::skip]
fn tags() -> Vec<Tag> {
    use Tag::*;
    vec![
        // Player, monsters, special entities. Triggers an interaction when players try to
        // move into them. These will have a Name tag. Often they will also have Scheduled,
        // and CanOpenDoor tags. NPCs will also have Behavior, Damage, Disposition,
        // Durability, Flees, Hearing, and Inventory tags.
        S("Character"),

        S("Player"),
        S("Doorman"), // TODO: might want to use a UniqueNpc for these
        S("Guard"),
        S("Icarium"),
        S("Rhulad"),
        S("Spectator"),

        // Present for objects that perform actions using the Scheduler.
        S("Scheduled"),

        // This is typically a base damage and is scaled by things like skill and strength.
        P("Damage", "i32"),

        // Percentage of strikes that'll do critical damage.
        P("Crit", "i32"),

        // Amount of time it takes to use an item. TODO: may also want to use this for base character movement speed
        P("Delay", "Time"),

        // Objects that a Character has picked up.
        P("Inventory", "Vec<Oid>"),

        // Used for Characters that start fleeing when their HPs is at the specified percent.
        P("Flees", "i32"), // TODO: should this be smarter? or maybe a second type of flee tag that considers both attacker and defender HPs

        // Scaling factor applied to the probability of responding to noise. 100 is no scaling,
        // 120 is 20% more likely, and 80 is 20% less likely.
        P("Hearing", "i32"),

        S("CanOpenDoor"),

        // The object is something that can be picked up and placed into a
        // Character's inventory.
        S("Portable"),

        // Can be used to dig through wood or stone structures (i.e. doors and
        // walls). Ineffective against metal.
        S("PickAxe"),

        // Description will have the sign's message.
        S("Sign"),

        S("EmpSword"),// TODO: do we want UniqueNPC and UniqueItem?

        // Used for objects that are the lowest layer in a Cell, e.g. grassy ground.
        // Note that this can be used for unusual objects such as a ballista. Will
        // have a Background tag.
        P("Terrain", "Terrain"),

        // Normally only used with Terrain.
        P("Background", "Color"),

        P("Disposition", "Disposition"),

        P("Behavior", "Behavior"),

        // Typically at zero durability an object will change somehow, e.g. a
        // door will become open or a character will die.
        P("Durability", "Durability"),

        // Used for some terrain objects, e.g. walls and doors.
        P("Material", "Material"),

        // Characters and portable objects all have names.
        P("Name", "&'static str"),

        // ---- Stats --------------------------------------------------------------------
        // These don't confer any extra abilities (that's skills). Stats merely allow you
        // to do more of what you can already do.

        // Multiplies damage according to how much stronger the character is than the min
        // weapon strength up to a maximum (so more strength only helps damage for heavy
        // weapons). Also eliminates penalties for wearing armor that is too heavy.
        P("Strength", "i32"),

        // Increases the chance of crits and acts as multiplier for dodge. Better for light
        // weapons and armor beause heavy weapons have a very small crit chance and heavy
        // armor significantly reduces dodge.
        P("Dexterity", "i32"),
    ]
}

fn capitilize(s: &str) -> String {
    let letters: Vec<char> = s.chars().collect();

    let mut result = String::new();
    for (i, c) in letters.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(*c);
    }
    result.to_uppercase()
}

fn generate_tag_file(out_dir: &str) -> Result<(), std::io::Error> {
    let dest_path = Path::new(&out_dir).join("tag.rs");
    let mut f = File::create(&dest_path)?;

    // Write out the Tag enum.
    writeln!(f, "#[derive(Clone, Debug, Eq, PartialEq)]")?;
    writeln!(f, "pub enum Tag {{")?;
    let tags = tags();
    for tag in &tags {
        match tag {
            Tag::S(name) => {
                writeln!(f, "    {name},")?;
            }
            Tag::P(name, arg) => {
                writeln!(f, "    {name}({arg}),")?;
            }
        }
    }
    writeln!(f, "}}\n")?;

    // Write out Tid constants for each tag.
    for (i, tag) in tags.iter().enumerate() {
        let name = match tag {
            Tag::S(name) => name,
            Tag::P(name, _) => name,
        };
        let name = capitilize(name);
        writeln!(f, "pub const {name}_ID: Tid = Tid({i});")?;
    }
    writeln!(f, "")?;

    // Write out the to_id function.
    writeln!(f, "impl Tag {{")?;
    writeln!(f, "    pub fn to_id(&self) -> Tid {{")?;
    writeln!(f, "        match self {{")?;
    for tag in &tags {
        let (name, suffix) = match tag {
            Tag::S(name) => (name, ""),
            Tag::P(name, _) => (name, "(_)"),
        };
        let uname = capitilize(name);
        writeln!(f, "            Tag::{name}{suffix} => {uname}_ID,")?;
    }
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f, "}}\n")?;

    // Write out the Display trait.
    writeln!(f, "impl fmt::Display for Tag {{")?;
    writeln!(f, "    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {{")?;
    writeln!(f, "        match self {{")?;
    for tag in &tags {
        let (name, suffix) = match tag {
            Tag::S(name) => (name, ""),
            Tag::P(name, _) => (name, "(_)"),
        };
        writeln!(f, "            Tag::{name}{suffix} => write!(f, \"{name}\"),")?;
    }
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f, "}}")?;

    Ok(())
}

fn generate_obj_file(out_dir: &str) -> Result<(), std::io::Error> {
    let dest_path = Path::new(&out_dir).join("obj.rs");
    let mut f = File::create(&dest_path)?;

    let tags = tags();
    writeln!(f, "impl Object {{")?;
    for tag in &tags {
        match tag {
            Tag::S(_) => (),
            Tag::P(name, arg) => {
                let lname = name.to_lowercase();
                if arg.contains('<') {
                    // If the argument is a collection then we want to return a reference
                    // to the value (and a mutable version).
                    writeln!(f, "    pub fn {lname}_value(&self) -> Option<&{arg}> {{")?;
                    writeln!(f, "        for candidate in &self.tags {{")?;
                    writeln!(f, "            if let Tag::{name}(value) = candidate {{")?;
                    writeln!(f, "                return Some(value);")?;
                    writeln!(f, "            }}")?;
                    writeln!(f, "        }}")?;
                    writeln!(f, "        None")?;
                    writeln!(f, "    }}\n")?;

                    writeln!(f, "    pub fn {lname}_value_mut(&mut self) -> Option<&mut {arg}> {{",)?;
                    writeln!(f, "        for candidate in &mut self.tags {{")?;
                    writeln!(f, "            if let Tag::{name}(value) = candidate {{")?;
                    writeln!(f, "                return Some(value);")?;
                    writeln!(f, "            }}")?;
                    writeln!(f, "        }}")?;
                    writeln!(f, "        None")?;
                    writeln!(f, "    }}\n")?;
                } else {
                    writeln!(f, "    pub fn {lname}_value(&self) -> Option<{arg}> {{")?;
                    writeln!(f, "        for candidate in &self.tags {{")?;
                    writeln!(f, "            if let Tag::{name}(value) = candidate {{")?;
                    writeln!(f, "                return Some(*value);")?;
                    writeln!(f, "            }}")?;
                    writeln!(f, "        }}")?;
                    writeln!(f, "        None")?;
                    writeln!(f, "    }}\n")?;
                }
            }
        }
    }
    writeln!(f, "}}")?;

    Ok(())
}

// See https://doc.rust-lang.org/cargo/reference/build-scripts.html.
fn main() -> Result<(), std::io::Error> {
    let out_dir = env::var("OUT_DIR").unwrap();
    generate_tag_file(&out_dir)?;
    generate_obj_file(&out_dir)?;
    // TODO: could use rustc-env=VAR=VALUE to embed a git hash into the exe

    Ok(())
}
