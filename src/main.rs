#[cfg(not(feature = "interpreter"))]
use rustyline::error::ReadlineError;
#[cfg(not(feature = "interpreter"))]
use rustyline::DefaultEditor;
use savan::lex;
use savan::nav::errors::{NavigatorError, Result};
use savan::nav::{facets::Facets, Navigator};
use std::fs::read_to_string;
use std::path::Path;
mod config;
mod interpreter;
mod modes;
mod reencode;
#[cfg(feature = "interpreter")]
use crate::config::PROMPT;
use crate::interpreter::Evaluate;
use crate::modes::Mode;

#[cfg(feature = "verbose")]
use std::time::Instant;

#[cfg(not(feature = "interpreter"))]
fn main() -> Result<()> {
    use crate::reencode::Selection;

    let mut input = std::env::args().skip(1);
    let arg = match input.next() {
        Some(s) => s,
        _ => {
            println!("error: expected input logic program");
            std::process::exit(-1)
        }
    };

    let args = input.collect::<Vec<_>>();
    let lp = read_to_string(Path::new(&arg)).map_err(|_| NavigatorError::None)?;

    println!(
        "{} v{}\n{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        config::WATERMARK
    );

    #[cfg(feature = "verbose")]
    println!("% startup started");
    #[cfg(feature = "verbose")]
    let start = Instant::now();
    let mut nav = Navigator::new(lp, args)?;
    let mut mode = Mode::GoalOriented(None::<usize>);
    let mut route = Vec::new();
    let mut facets = nav
        .facet_inducing_atoms(route.iter())
        .ok_or(NavigatorError::None)?
        .iter()
        .map(|f| lex::repr(*f))
        .collect::<Vec<_>>();
    let mut selection = Selection::new();
    #[cfg(feature = "verbose")]
    println!("% startup elapsed: {:?}", start.elapsed());

    let mut rl = DefaultEditor::new().map_err(|_| NavigatorError::None)?;
    loop {
        match rl.readline(crate::config::PROMPT) {
            Ok(line) => {
                if let Err(err) = rl.add_history_entry(line.as_str()) {
                    eprintln!("ReadlineError: {:?}", err);
                }

                mode.command(
                    line.replace("_", " "),
                    &mut nav,
                    &mut facets,
                    &mut route,
                    &mut selection,
                )?;
            }
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("ReadlineError: {:?}", err);
            }
        }
    }

    Ok(())
}

#[cfg(feature = "interpreter")]
fn main() -> Result<()> {
    let mut input = std::env::args().skip(1);
    let arg = match input.next() {
        Some(s) => s,
        _ => {
            println!("error: expected input logic program");
            std::process::exit(-1)
        }
    };

    let mut args = input.collect::<Vec<_>>();
    let lp = read_to_string(Path::new(&arg)).map_err(|_| NavigatorError::None)?;

    println!(
        "{} v{}\n{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        config::WATERMARK,
    );

    let script =
        read_to_string(Path::new(args.last().unwrap())).map_err(|_| NavigatorError::None)?;
    args.pop();

    #[cfg(feature = "verbose")]
    println!("% startup started");
    #[cfg(feature = "verbose")]
    let start = Instant::now();
    let mut nav = Navigator::new(lp, args)?;
    let mut mode = Mode::GoalOriented(None::<usize>);
    let mut route = Vec::new();
    let mut facets = nav
        .facet_inducing_atoms(route.iter())
        .ok_or(NavigatorError::None)?
        .iter()
        .map(|f| lex::repr(*f))
        .collect::<Vec<_>>();
    let mut selection = Selection::new();
    #[cfg(feature = "verbose")]
    println!("% startup elapsed: {:?}", start.elapsed());

    for line in script.lines() {
        println!("{PROMPT}{line}");
        mode.command(
            line.replace("_", " "),
            &mut nav,
            &mut facets,
            &mut route,
            &mut selection,
        )?;
    }

    return Ok(());
}
