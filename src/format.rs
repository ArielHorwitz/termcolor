use clap::Parser;

const CODE_START: &str = "\x1b[";
const CODE_END: &str = "m";
const RESET: &str = "\x1b[m";
const AFTER_LONG_HELP: &str = "\x1b[1;4mColors (use uppercase for brighter color):\x1b[0m
blac(k), (w)hite, (r)ed, (g)reen, (b)lue, (y)ellow, (c)yan, (m)agenta

\x1b[1;4mFormatting options\x1b[0m:
(b)old, (d)im, (u)nderline, (i)talic, (s)trikethrough

\x1b[1;4mStyles\x1b[0m:
ok, notice, error, warn, info, debug";

#[derive(Debug, Parser)]
#[clap(about = "Format text for ANSI terminal.")]
#[clap(author = "https://ariel.ninja")]
#[clap(hide_possible_values = true)]
#[clap(version)]
#[clap(after_long_help = AFTER_LONG_HELP)]
pub struct Args {
    /// Text to format
    #[arg()]
    text: Vec<String>,
    /// Premade style
    #[arg(short = 's', long, conflicts_with_all = ["foreground", "background", "options"])]
    style: Option<Style>,
    /// Foreground color
    #[arg(short = 'f', long)]
    foreground: Option<Color>,
    /// Background color
    #[arg(short = 'b', long)]
    background: Option<Color>,
    /// Formatting options
    #[arg(short = 'o', long, value_delimiter = ',')]
    options: Vec<FormattingOption>,
    /// Reset formatting before text
    #[arg(short = 'R', long)]
    reset: bool,
    /// Do not reset formatting after text
    #[arg(short = 'r', long)]
    no_reset: bool,
    /// Do not print newline
    #[arg(short = 'n', long)]
    no_newline: bool,
}

/// Formatting options
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
enum FormattingOption {
    #[clap(alias = "b")]
    Bold,
    #[clap(alias = "d")]
    Dim,
    #[clap(alias = "u")]
    Underline,
    #[clap(alias = "i")]
    Inverted,
    #[clap(alias = "s")]
    Strikethrough,
}

/// Color options
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
enum Color {
    #[clap(alias = "k")]
    Black,
    #[clap(alias = "w")]
    White,
    #[clap(alias = "r")]
    Red,
    #[clap(alias = "g")]
    Green,
    #[clap(alias = "b")]
    Blue,
    #[clap(alias = "y")]
    Yellow,
    #[clap(alias = "c")]
    Cyan,
    #[clap(alias = "m")]
    Magenta,
    #[clap(alias = "K", alias = "BLACK")]
    BrightBlack,
    #[clap(alias = "W", alias = "WHITE")]
    BrightWhite,
    #[clap(alias = "R", alias = "RED")]
    BrightRed,
    #[clap(alias = "G", alias = "GREEN")]
    BrightGreen,
    #[clap(alias = "B", alias = "BLUE")]
    BrightBlue,
    #[clap(alias = "Y", alias = "YELLOW")]
    BrightYellow,
    #[clap(alias = "C", alias = "CYAN")]
    BrightCyan,
    #[clap(alias = "M", alias = "MAGENTA")]
    BrightMagenta,
}

/// Color options
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
enum Style {
    Ok,
    Notice,
    Error,
    Warn,
    Info,
    Debug,
}

pub fn format(mut args: Args) -> String {
    // Premade Style
    args = apply_style(args);

    // Text formatting
    let mut prop_codes = Vec::new();
    if let Some(fg) = args.foreground {
        prop_codes.push((30 + get_color_code_digit(fg)).to_string());
    }
    if let Some(bg) = args.background {
        prop_codes.push((40 + get_color_code_digit(bg)).to_string());
    }
    for option in &args.options {
        prop_codes.push(get_format_code(option).to_string());
    }

    // Formatted text
    let text = args.text.join(" ");
    let mut result = if !prop_codes.is_empty() {
        let prop_codes = prop_codes.join(";");
        format!("{CODE_START}{prop_codes}{CODE_END}{text}")
    } else {
        text
    };

    // Pre-text formatting
    if args.reset {
        result = format!("{RESET}{result}")
    }
    // Post-text formatting
    if !args.no_reset {
        result.push_str(RESET);
    };
    if !args.no_newline {
        result.push('\n');
    };

    result
}

fn apply_style(mut args: Args) -> Args {
    if let Some(style) = args.style {
        args.background = None;
        args.foreground = None;
        args.options = Vec::new();
        match style {
            Style::Ok => {
                args.foreground = Some(Color::Green);
            }
            Style::Notice => {
                args.foreground = Some(Color::Magenta);
            }
            Style::Error => {
                args.foreground = Some(Color::Red);
            }
            Style::Warn => {
                args.foreground = Some(Color::Yellow);
            }
            Style::Info => {
                args.foreground = Some(Color::Cyan);
            }
            Style::Debug => {
                args.background = Some(Color::Cyan);
                args.foreground = Some(Color::Black);
                args.options.push(FormattingOption::Dim);
            }
        }
    }
    args
}

fn get_format_code(option: &FormattingOption) -> u8 {
    match option {
        FormattingOption::Bold => 1,
        FormattingOption::Dim => 2,
        FormattingOption::Underline => 4,
        FormattingOption::Inverted => 7,
        FormattingOption::Strikethrough => 9,
    }
}

fn get_color_code_digit(color: Color) -> u8 {
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::White => 7,
        Color::BrightBlack => 60,
        Color::BrightRed => 61,
        Color::BrightGreen => 62,
        Color::BrightYellow => 63,
        Color::BrightBlue => 64,
        Color::BrightMagenta => 65,
        Color::BrightCyan => 66,
        Color::BrightWhite => 67,
    }
}
