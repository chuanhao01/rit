use std::{
    collections::HashMap,
    env,
    fs::{create_dir, File},
    io,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use configparser::ini::Ini;
use homedir::get_my_home;

#[derive(Debug, Parser)]
#[command(name = "rit")]
#[command(about = "Rust git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
}

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);
}
