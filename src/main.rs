use clap::Parser;

mod args;
mod interface;
mod metadata;

fn main() {
    let args = args::Arguments::parse();
    interface::init();
}
