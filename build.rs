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

// See https://doc.rust-lang.org/cargo/reference/build-scripts.html.
fn main() -> Result<(), std::io::Error> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tag.rs");
    let mut f = File::create(&dest_path)?;

    // Write out the Tag enum.
    f.write_all(b"#[derive(Clone, Debug, Eq, PartialEq)]\n")?;
    f.write_all(b"pub enum Tag {\n")?;
    let tags = tags();
    for tag in &tags {
        match tag {
            Tag::S(name) => {
                f.write_all(format!("    {name},\n").as_bytes())?;
            }
            Tag::P(name, arg) => {
                f.write_all(format!("    {name}({arg}),\n").as_bytes())?;
            }
        }
    }
    f.write_all(b"}\n\n")?;

    // Write out Tid constants for each tag.
    for (i, tag) in tags.iter().enumerate() {
        let name = match tag {
            Tag::S(name) => name,
            Tag::P(name, _) => name,
        };
        let name = capitilize(name);
        f.write_all(format!("pub const {name}_ID: Tid = Tid({i});\n").as_bytes())?;
    }
    f.write_all(b"\n")?;

    // Write out the to_id function.
    f.write_all(b"impl Tag {\n")?;
    f.write_all(b"    pub fn to_id(&self) -> Tid {\n")?;
    f.write_all(b"        match self {\n")?;
    for tag in &tags {
        let (name, suffix) = match tag {
            Tag::S(name) => (name, ""),
            Tag::P(name, _) => (name, "(_)"),
        };
        let uname = capitilize(name);
        f.write_all(format!("            Tag::{name}{suffix} => {uname}_ID,\n").as_bytes())?;
    }
    f.write_all(b"        }\n")?;
    f.write_all(b"    }\n")?;
    f.write_all(b"}\n\n")?;

    // Write out the Display trait.
    f.write_all(b"impl fmt::Display for Tag {\n")?;
    f.write_all(b"    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {\n")?;
    f.write_all(b"        match self {\n")?;
    for tag in &tags {
        let (name, suffix) = match tag {
            Tag::S(name) => (name, ""),
            Tag::P(name, _) => (name, "(_)"),
        };
        f.write_all(format!("            Tag::{name}{suffix} => write!(f, \"{name}\"),\n").as_bytes())?;
    }
    f.write_all(b"        }\n")?;
    f.write_all(b"    }\n")?;
    f.write_all(b"}\n")?;

    // TODO:
    // write out stuff like behavior_value (probably into a separate file)
    // get rid of the old value trait impls
    // write out how this was generated?
    // write out a timestamp?

    Ok(())
}
