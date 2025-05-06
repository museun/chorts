use crate::Error;

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Style {
    #[serde(default)]
    pub color: Option<Color>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub dimmed: bool,
}

#[cfg(feature = "anstyle")]
impl From<Style> for ::anstyle::Style {
    fn from(value: Style) -> Self {
        let mut this = if let Some(color) = value.color {
            Self::new().fg_color(Some(color.into()))
        } else {
            Self::new()
        };
        type Apply = fn(::anstyle::Style) -> ::anstyle::Style;
        for (val, apply) in [
            (value.bold, Self::bold as Apply),
            (value.italic, Self::italic),
            (value.underline, Self::underline),
            (value.dimmed, Self::dimmed),
        ] {
            if val {
                this = apply(this)
            }
        }
        this
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Black,         // = 0,
    Red,           // = 1,
    Green,         // = 2,
    Yellow,        // = 3,
    Blue,          // = 4,
    Magenta,       // = 5,
    Cyan,          // = 6,
    White,         // = 7,
    BrightBlack,   // = 8,
    BrightRed,     // = 9,
    BrightGreen,   // = 10,
    BrightYellow,  // = 11,
    BrightBlue,    // = 12,
    BrightMagenta, // = 13,
    BrightCyan,    // = 14,
    BrightWhite,   // = 15,
    Rgb(u8, u8, u8),
}

#[cfg(feature = "anstyle")]
impl From<Color> for ::anstyle::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => Self::Ansi(anstyle::AnsiColor::Black),
            Color::Red => Self::Ansi(anstyle::AnsiColor::Red),
            Color::Green => Self::Ansi(anstyle::AnsiColor::Green),
            Color::Yellow => Self::Ansi(anstyle::AnsiColor::Yellow),
            Color::Blue => Self::Ansi(anstyle::AnsiColor::Blue),
            Color::Magenta => Self::Ansi(anstyle::AnsiColor::Magenta),
            Color::Cyan => Self::Ansi(anstyle::AnsiColor::Cyan),
            Color::White => Self::Ansi(anstyle::AnsiColor::White),
            Color::BrightBlack => Self::Ansi(anstyle::AnsiColor::BrightBlack),
            Color::BrightRed => Self::Ansi(anstyle::AnsiColor::BrightRed),
            Color::BrightGreen => Self::Ansi(anstyle::AnsiColor::BrightGreen),
            Color::BrightYellow => Self::Ansi(anstyle::AnsiColor::BrightYellow),
            Color::BrightBlue => Self::Ansi(anstyle::AnsiColor::BrightBlue),
            Color::BrightMagenta => Self::Ansi(anstyle::AnsiColor::BrightMagenta),
            Color::BrightCyan => Self::Ansi(anstyle::AnsiColor::BrightCyan),
            Color::BrightWhite => Self::Ansi(anstyle::AnsiColor::BrightWhite),
            Color::Rgb(r, g, b) => Self::Rgb(anstyle::RgbColor(r, g, b)),
        }
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Black => write!(f, "Black"),
            Self::Red => write!(f, "Red"),
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Blue => write!(f, "Blue"),
            Self::Magenta => write!(f, "Magenta"),
            Self::Cyan => write!(f, "Cyan"),
            Self::White => write!(f, "White"),
            Self::BrightBlack => write!(f, "BrightBlack"),
            Self::BrightRed => write!(f, "BrightRed"),
            Self::BrightGreen => write!(f, "BrightGreen"),
            Self::BrightYellow => write!(f, "BrightYellow"),
            Self::BrightBlue => write!(f, "BrightBlue"),
            Self::BrightMagenta => write!(f, "BrightMagenta"),
            Self::BrightCyan => write!(f, "BrightCyan"),
            Self::BrightWhite => write!(f, "BrightWhite"),
            Self::Rgb(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        <String>::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl std::str::FromStr for Color {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Error::EmptyColor);
        }

        if let Some(input) = input.strip_prefix('#') {
            return match input.len() {
                3 => u16::from_str_radix(input, 16)
                    .map_err(|_| Error::InvalidHexColor(input.to_string()))
                    .map(Self::from_u16),
                6 => u32::from_str_radix(input, 16)
                    .map_err(|_| Error::InvalidHexColor(input.to_string()))
                    .map(Self::from_u32),
                _ => Err(Error::InvalidHexColor(input.to_string())),
            };
        }

        if input.starts_with("rgb(") && input.ends_with(")") {
            let input = input[4..input.len() - 1].trim();
            if input.is_empty() {
                return Err(Error::EmptyColor);
            }

            let mut iter = input.split_terminator(',').map(|s| s.trim().parse());
            let r = iter
                .next()
                .and_then(Result::ok)
                .ok_or_else(|| Error::InvalidRgbColor(input.to_string()))?;
            let g = iter
                .next()
                .and_then(Result::ok)
                .ok_or_else(|| Error::InvalidRgbColor(input.to_string()))?;
            let b = iter
                .next()
                .and_then(Result::ok)
                .ok_or_else(|| Error::InvalidRgbColor(input.to_string()))?;
            return Ok(Self::Rgb(r, g, b));
        }

        const TABLE: [(&[&str], Color); 16] = [
            (&["Black"], Color::Black),
            (&["Red"], Color::Red),
            (&["Green"], Color::Green),
            (&["Yellow"], Color::Yellow),
            (&["Blue"], Color::Blue),
            (&["Magenta"], Color::Magenta),
            (&["Cyan"], Color::Cyan),
            (&["White"], Color::White),
            (&["BrightBlack", "Bright Black"], Color::BrightBlack),
            (&["BrightRed", "Bright Red"], Color::BrightRed),
            (&["BrightGreen", "Bright Green"], Color::BrightGreen),
            (&["BrightYellow", "Bright Yellow"], Color::BrightYellow),
            (&["BrightBlue", "Bright Blue"], Color::BrightBlue),
            (&["BrightMagenta", "Bright Magenta"], Color::BrightMagenta),
            (&["BrightCyan", "Bright Cyan"], Color::BrightCyan),
            (&["BrightWhite", "Bright White"], Color::BrightWhite),
        ];

        for (opts, kind) in TABLE {
            for opt in opts {
                if input.eq_ignore_ascii_case(opt) {
                    return Ok(kind);
                }
            }
        }

        Err(Error::InvalidColor(input.to_string()))
    }
}

impl Color {
    pub const fn from_u32(color: u32) -> Self {
        let [_, r, g, b] = color.to_be_bytes();
        Self::Rgb(r, g, b)
    }

    pub const fn from_u16(color: u16) -> Self {
        let offset = if ((color >> 12) & ((1 << 4) - 1)) == 0 {
            4
        } else {
            0
        };

        let r = ((color >> (12 - offset)) & 0xF) as u8;
        let g = ((color >> (8 - offset)) & 0xF) as u8;
        let b = ((color >> (4 - offset)) & 0xF) as u8;

        Self::Rgb((r << 4) | r, (g << 4) | g, (b << 4) | b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_from_u32() {
        for (input, expected) in [
            (0xFF0000, Color::Rgb(0xFF, 0x00, 0x00)),
            (0x00FF00, Color::Rgb(0x00, 0xFF, 0x00)),
            (0x0000FF, Color::Rgb(0x00, 0x00, 0xFF)),
            (0x112233, Color::Rgb(0x11, 0x22, 0x33)),
            (0x332211, Color::Rgb(0x33, 0x22, 0x11)),
            (0x00F0F0F, Color::Rgb(0x0F, 0x0F, 0x0F)),
            (0x0F0F0F0, Color::Rgb(0xF0, 0xF0, 0xF0)),
        ] {
            assert_eq!(Color::from_u32(input), expected);
        }
    }

    #[test]
    fn color_from_u16() {
        for (input, expected) in [
            (0xF00, Color::Rgb(0xFF, 0x00, 0x00)),
            (0x0F0, Color::Rgb(0x00, 0xFF, 0x00)),
            (0x00F, Color::Rgb(0x00, 0x00, 0xFF)),
            (0x123, Color::Rgb(0x11, 0x22, 0x33)),
            (0x321, Color::Rgb(0x33, 0x22, 0x11)),
            (0x0123, Color::Rgb(0x11, 0x22, 0x33)),
            (0x0321, Color::Rgb(0x33, 0x22, 0x11)),
        ] {
            assert_eq!(Color::from_u16(input), expected);
        }
    }

    #[test]
    fn color_from_str() {
        for (input, output) in [
            ("#112233", Color::Rgb(0x11, 0x22, 0x33)),
            ("#123", Color::Rgb(0x11, 0x22, 0x33)),
            ("#FF0000", Color::Rgb(0xFF, 0x00, 0x00)),
            ("#F00", Color::Rgb(0xFF, 0x00, 0x00)),
            ("#0000FF", Color::Rgb(0x00, 0x00, 0xFF)),
            ("#00F", Color::Rgb(0x00, 0x00, 0xFF)),
            //
            ("rgb(10, 20, 30)", Color::Rgb(10, 20, 30)),
            ("rgb(30, 20, 10)", Color::Rgb(30, 20, 10)),
            //
            ("Black", Color::Black),
            ("Red", Color::Red),
            ("Green", Color::Green),
            ("Yellow", Color::Yellow),
            ("Blue", Color::Blue),
            ("Magenta", Color::Magenta),
            ("Cyan", Color::Cyan),
            ("White", Color::White),
            ("BrightBlack", Color::BrightBlack),
            ("BrightRed", Color::BrightRed),
            ("BrightGreen", Color::BrightGreen),
            ("BrightYellow", Color::BrightYellow),
            ("BrightBlue", Color::BrightBlue),
            ("BrightMagenta", Color::BrightMagenta),
            ("BrightCyan", Color::BrightCyan),
            ("BrightWhite", Color::BrightWhite),
            //
            ("Bright Black", Color::BrightBlack),
            ("Bright Red", Color::BrightRed),
            ("Bright Green", Color::BrightGreen),
            ("Bright Yellow", Color::BrightYellow),
            ("Bright Blue", Color::BrightBlue),
            ("Bright Magenta", Color::BrightMagenta),
            ("Bright Cyan", Color::BrightCyan),
            ("Bright White", Color::BrightWhite),
            //
            ("black", Color::Black),
            ("red", Color::Red),
            ("green", Color::Green),
            ("yellow", Color::Yellow),
            ("blue", Color::Blue),
            ("magenta", Color::Magenta),
            ("cyan", Color::Cyan),
            ("white", Color::White),
            ("brightblack", Color::BrightBlack),
            ("brightred", Color::BrightRed),
            ("brightgreen", Color::BrightGreen),
            ("brightyellow", Color::BrightYellow),
            ("brightblue", Color::BrightBlue),
            ("brightmagenta", Color::BrightMagenta),
            ("brightcyan", Color::BrightCyan),
            ("brightwhite", Color::BrightWhite),
            //
            ("bright black", Color::BrightBlack),
            ("bright red", Color::BrightRed),
            ("bright green", Color::BrightGreen),
            ("bright yellow", Color::BrightYellow),
            ("bright blue", Color::BrightBlue),
            ("bright magenta", Color::BrightMagenta),
            ("bright cyan", Color::BrightCyan),
            ("bright white", Color::BrightWhite),
        ] {
            assert_eq!(
                input.parse::<Color>().unwrap(),
                output,
                "input: {input}, output: {output}"
            );
        }
    }
}
