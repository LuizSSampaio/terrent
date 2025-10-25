use clap::Parser;

mod args;
mod metadata;

fn main() {
    let args = args::Arguments::parse();
}
