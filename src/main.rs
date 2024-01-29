use clap::Parser;

const RESET: &str = "\x1b[m";
const FG_BLACK: &str = "\x1b[38;2;0;0;0m";
const FG_GREY: &str = "\x1b[38;5;250m";

#[derive(Debug, Parser)]
#[clap(about = "Display terminal colors")]
#[clap(author = "https://ariel.ninja")]
struct Args {
    /// Hue resolution
    #[arg(short = 'H', long, default_value_t = 16)]
    hues: u8,
    /// Value resolution
    #[arg(short = 'V', long, default_value_t = 4)]
    values: u8,
    /// Saturation resolution
    #[arg(short = 'S', long, default_value_t = 4)]
    saturations: u8,
    /// Resolution (overwrites both value and saturation resolutions)
    #[arg(short, long)]
    resolution: Option<u8>,
    /// Offset hue in degrees
    #[arg(short, long, default_value_t = 0.0)]
    offset: f64,
    /// Display options
    #[arg(short, long, value_enum, default_value_t = DisplayOptions::Rgb)]
    display: DisplayOptions,
    /// Dark color threshold
    #[arg(long, default_value_t = 50.0)]
    dark: f64,
    /// Dark color factor
    #[arg(short = 'D', long, default_value_t = 5.0)]
    dark_factor: f64,
    /// Show legend (hue, saturation, and value)
    #[arg(short, long)]
    legend: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
enum DisplayOptions {
    /// RGB
    Rgb,
    /// ANSI color codes
    Ansi,
    /// Luminosity (according to EIC-1931)
    Lum,
    /// none
    None,
}

fn main() {
    let mut args = Args::parse();
    if let Some(resolution) = args.resolution {
        args.values = resolution;
        args.saturations = resolution;
    }
    let hues = range(args.hues + 1, 0, 1, args.offset / 360.0);
    let values = range(args.values + 2, 1, 1, 0.0);
    let mut saturations = range(args.saturations + 2, 1, 1, 0.0);
    saturations.reverse();
    let is_table = !values.is_empty() || !saturations.is_empty();
    let legend = args.legend && args.display != DisplayOptions::Ansi;
    if legend && is_table {
        for v in &values {
            print!(" {:>3}% v  ", (v * 100.0).round());
        }
        print!(" val/sat ");
        for s in &saturations {
            print!(" {:>3}% s  ", (s * 100.0).round());
        }
        println!();
    }
    for h in hues {
        values.iter().for_each(|v| {
            Color::from_hsv(h, 1.0, *v).print(args.display, args.dark, args.dark_factor);
        });
        Color::from_hsv(h, 1.0, 1.0).print(args.display, args.dark, args.dark_factor);
        saturations.iter().for_each(|s| {
            Color::from_hsv(h, *s, 1.0).print(args.display, args.dark, args.dark_factor);
        });
        if legend {
            print!(" hue: {}", (h * 360.0).round());
        }
        println!();
    }
    if let DisplayOptions::Ansi = args.display {
        // Basic palette
        for i in 0..16 {
            if i % 8 == 0 {
                println!()
            }
            let foreground = if i == 0 { FG_GREY } else { FG_BLACK };
            let background = format!("\x1b[48;5;{i}m");
            print!("{background}{foreground}{i:^9}{RESET}");
        }
        println!();
        // Greyscale
        for i in 232..255 {
            if (i - 232) % 8 == 0 {
                println!()
            }
            let foreground = if i <= 237 { FG_GREY } else { FG_BLACK };
            let background = format!("\x1b[48;5;{i}m");
            print!("{background}{foreground}{i:^9}{RESET}");
        }
        println!();
    }
}

fn range(resolution: u8, truncate_head: u8, truncate_tail: u8, offset: f64) -> Vec<f64> {
    if resolution
        .saturating_sub(truncate_head)
        .saturating_sub(truncate_tail)
        == 0
    {
        return Vec::new();
    }
    let factor = 1.0 / f64::from(resolution.saturating_sub(1));
    (truncate_head..resolution.saturating_sub(truncate_tail))
        .map(|i| (f64::from(i) * factor + offset) % 1.0)
        .collect()
}

#[derive(Debug, Copy, Clone)]
struct Color(f64, f64, f64);

impl Color {
    fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let h = h * 360.0;

        let c = v * s;
        let x = c * (1.0 - f64::abs((h / 60.0) % 2.0 - 1.0));
        let m = v - c;
        let (r_, g_, b_) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        let r = r_ + m;
        let g = g_ + m;
        let b = b_ + m;
        Self(r, g, b)
    }

    fn as_bytes(&self) -> (u8, u8, u8) {
        (
            (self.0 * 255.0).round() as u8,
            (self.1 * 255.0).round() as u8,
            (self.2 * 255.0).round() as u8,
        )
    }

    fn eic_luminosity(&self) -> f64 {
        // let x = 0.4124 * self.0 + 0.3576 * self.1 + 0.1805 * self.2;
        0.2126 * self.0 + 0.7152 * self.1 + 0.0722 * self.2
        // let z = 0.0193 * self.0 + 0.1192 * self.1 + 0.9505 * self.2;
    }

    fn nearest_ansi_color_code(&self) -> u8 {
        let (r, g, b) = self.as_bytes();
        let r = (r / 32).min(5);
        let g = (g / 32).min(5);
        let b = (b / 32).min(5);
        16 + 36 * r + 6 * g + b
    }

    fn display_hex(&self) -> String {
        let (r, g, b) = self.as_bytes();
        format!("{r:02X}{g:02X}{b:02X}")
    }

    fn bg(&self) -> String {
        let (r, g, b) = self.as_bytes();
        format!("\x1b[48;2;{r};{g};{b}m")
    }

    fn fg(&self) -> String {
        let (r, g, b) = self.as_bytes();
        format!("\x1b[38;2;{r};{g};{b}m")
    }

    fn print(&self, display: DisplayOptions, dark_threshold: f64, dark_factor: f64) {
        let luminosity = self.eic_luminosity();
        let dark = dark_threshold / 100.0;
        let fgv = if luminosity > dark {
            (1.0 - luminosity).powf(dark_factor) // bright color, dark text
        } else {
            luminosity.powf(1.0 / dark_factor) // dark color, bright text
        };
        let foreground = Color::from_hsv(0.0, 0.0, fgv).fg();
        let (background, text) = match display {
            DisplayOptions::Ansi => {
                let color_code = self.nearest_ansi_color_code();
                let background = format!("\x1b[48;5;{}m", color_code);
                let text = format!("{:^6}", color_code);
                (background, text)
            }
            DisplayOptions::Rgb => (self.bg(), self.display_hex()),
            DisplayOptions::Lum => (self.bg(), format!("{:>3}%", (luminosity * 100.0).round())),
            DisplayOptions::None => (self.bg(), String::new()),
        };
        print!("{background}{foreground}{text:^9}{RESET}");
    }
}
