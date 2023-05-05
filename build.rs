use std::env;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/args.rs");

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut command = Args::command();
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        generate_to(shell, &mut command, "chwd", &out_path).expect("Couldn't generate completion!");
    }
}
