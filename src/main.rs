mod app;
mod cli;
mod domain;
mod infra;
mod render;

use std::ffi::OsString;

use clap::Parser;

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();

    let exit_code = match cli::Cli::try_parse_from(&args) {
        Ok(cli) => app::run(cli),
        Err(error) => cli::render_parse_error(&args, error),
    };

    std::process::exit(exit_code);
}
