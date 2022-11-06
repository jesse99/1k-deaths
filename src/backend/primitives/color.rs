use std::fmt;

// These are the X11 color names (from http://cng.seas.rochester.edu/CNG/docs/x11color.html).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    LightPink,
    Pink,
    Crimson,
    LavenderBlush,
    PaleVioletRed,
    HotPink,
    DeepPink,
    MediumVioletRed,
    Orchid,
    Thistle,
    Plum,
    Violet,
    Magenta,
    Fuchsia,
    DarkMagenta,
    Purple,
    MediumOrchid,
    DarkViolet,
    DarkOrchid,
    Indigo,
    BlueViolet,
    MediumPurple,
    MediumSlateBlue,
    SlateBlue,
    DarkSlateBlue,
    Lavender,
    GhostWhite,
    Blue,
    MediumBlue,
    MidnightBlue,
    DarkBlue,
    Navy,
    RoyalBlue,
    CornflowerBlue,
    LightSteelBlue,
    LightSlateGray,
    SlateGray,
    DodgerBlue,
    AliceBlue,
    SteelBlue,
    LightSkyBlue,
    SkyBlue,
    DeepSkyBlue,
    LightBlue,
    PowderBlue,
    CadetBlue,
    Azure,
    LightCyan,
    PaleTurquoise,
    Cyan,
    Aqua,
    DarkTurquoise,
    DarkSlateGray,
    DarkCyan,
    Teal,
    MediumTurquoise,
    LightSeaGreen,
    Turquoise,
    Aquamarine,
    MediumAquamarine,
    MediumSpringGreen,
    MintCream,
    SpringGreen,
    MediumSeaGreen,
    SeaGreen,
    Honeydew,
    LightGreen,
    PaleGreen,
    DarkSeaGreen,
    LimeGreen,
    Lime,
    ForestGreen,
    Green,
    DarkGreen,
    Chartreuse,
    LawnGreen,
    GreenYellow,
    DarkOliveGreen,
    YellowGreen,
    OliveDrab,
    Beige,
    LightGoldenrodYellow,
    Ivory,
    LightYellow,
    Yellow,
    Olive,
    DarkKhaki,
    LemonChiffon,
    PaleGoldenrod,
    Khaki,
    Gold,
    Cornsilk,
    Goldenrod,
    DarkGoldenrod,
    FloralWhite,
    OldLace,
    Wheat,
    Moccasin,
    Orange,
    PapayaWhip,
    BlanchedAlmond,
    NavajoWhite,
    AntiqueWhite,
    Tan,
    BurlyWood,
    Bisque,
    DarkOrange,
    Linen,
    Peru,
    PeachPuff,
    SandyBrown,
    Chocolate,
    SaddleBrown,
    Seashell,
    Sienna,
    LightSalmon,
    Coral,
    OrangeRed,
    DarkSalmon,
    Tomato,
    MistyRose,
    Salmon,
    Snow,
    LightCoral,
    RosyBrown,
    IndianRed,
    Red,
    Brown,
    FireBrick,
    DarkRed,
    Maroon,
    White,
    WhiteSmoke,
    Gainsboro,
    LightGrey,
    Silver,
    DarkGray,
    Gray,
    DimGray,
    Black,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
