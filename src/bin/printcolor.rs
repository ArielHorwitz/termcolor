use clap::Parser;
use termcolors::format;

fn main() {
    let args = format::Args::parse();
    print!("{}", format::format(args));
}
