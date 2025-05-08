use regex::Regex;
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
mod is_facet;
mod modes;
mod significance;
mod wfc;
#[cfg(feature = "interpreter")]
use crate::config::PROMPT;
use crate::interpreter::Evaluate;
use crate::modes::Mode;

#[cfg(not(feature = "interpreter"))]
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
    let mut facets_at_startup = true;
    let mut learned_that_at_startup = false;
    if args.contains(&"--f".to_owned()) {
        facets_at_startup = false;
        let i = unsafe {
            args.iter()
                .position(|x| *x == "--f".to_owned())
                .unwrap_unchecked()
        };
        args.remove(i);
    }
    if args.contains(&"--l".to_owned()) {
        learned_that_at_startup = true;
        let i = unsafe {
            args.iter()
                .position(|x| *x == "--l".to_owned())
                .unwrap_unchecked()
        };
        args.remove(i);
    }
    let lp = read_to_string(Path::new(&arg)).map_err(|_| NavigatorError::None)?;

    let re = Regex::new(r#config::FILTER_KEYWORD).unwrap();
    let filter_re = match lp.lines().last() {
        Some(x) => {
            if re.is_match(x) {
                let s = &x.replace(config::FILTER_KEYWORD, "");
                Regex::new(r#s).map_err(|_| NavigatorError::None)?
            } else {
                let s = "";
                Regex::new(r#s).map_err(|_| NavigatorError::None)?
            }
        }
        _ => todo!(),
    };

    println!(
        "{} v{} (repl)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let mut nav = Navigator::new(lp.clone(), args.clone())?;
    let mut mode = Mode::GoalOriented(None::<usize>);
    let mut atoms = nav
        .atoms()
        .filter(|a| filter_re.is_match(a))
        .collect::<Vec<String>>();
    let mut route = Vec::new();
    let mut cnf = Vec::new();

    let mut facets = if facets_at_startup {
        match learned_that_at_startup {
            false => nav
                .facet_inducing_atoms(route.iter())
                .ok_or(NavigatorError::None)?
                .into_iter()
                .map(lex::repr)
                .collect::<Vec<_>>(),
            _ => nav
                .learned_that(&atoms, &route, None)
                .ok_or(NavigatorError::None)?,
        }
    } else {
        vec![]
    };

    let mut rl = DefaultEditor::new().map_err(|_| NavigatorError::None)?;

    //for a in nav.atoms() {
    //    if let Err(err) = rl.add_history_entry(a.as_str()) {
    //        eprintln!("ReadlineError: {:?}", err);
    //    }
    //}

    loop {
        match rl.readline(crate::config::PROMPT) {
            Ok(line) => {
                if let Err(err) = rl.add_history_entry(line.as_str()) {
                    eprintln!("ReadlineError: {:?}", err);
                }

                mode.command(
                    line,
                    &mut nav,
                    &mut atoms,
                    &mut facets,
                    &mut route,
                    &mut cnf,
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
        "{} v{} (interpreter)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    // NOTE: script has to be last argument
    let script =
        read_to_string(Path::new(args.last().unwrap())).map_err(|_| NavigatorError::None)?;
    args.pop();

    let mut facets_at_startup = true;
    let mut learned_that_at_startup = false;
    if args.contains(&"--f".to_owned()) {
        facets_at_startup = false;
        let i = unsafe {
            args.iter()
                .position(|x| *x == "--f".to_owned())
                .unwrap_unchecked()
        };
        args.remove(i);
    }
    if args.contains(&"--l".to_owned()) {
        learned_that_at_startup = true;
        let i = unsafe {
            args.iter()
                .position(|x| *x == "--l".to_owned())
                .unwrap_unchecked()
        };
        args.remove(i);
    }

    let clp = is_facet::copy_program(lp.clone());
    let mut _nav = Navigator::new(format!("{lp}\n{clp}"), args.clone())?;

    let re = Regex::new(r#config::FILTER_KEYWORD).unwrap();
    let filter_re = match lp.lines().last() {
        Some(x) => {
            if re.is_match(x) {
                let s = &x.replace(config::FILTER_KEYWORD, "");
                Regex::new(r#s).map_err(|_| NavigatorError::None)?
            } else {
                let s = "";
                Regex::new(r#s).map_err(|_| NavigatorError::None)?
            }
        }
        _ => todo!(),
    };

    let mut nav = Navigator::new(lp, args)?;
    let mut mode = Mode::GoalOriented(None::<usize>);

    let mut atoms = nav
        .atoms()
        .filter(|a| filter_re.is_match(a))
        .collect::<Vec<String>>();
    let mut route = Vec::new();
    let mut ctx = Vec::new();
    let mut facets = if facets_at_startup {
        match learned_that_at_startup {
            false => nav
                .facet_inducing_atoms(route.iter())
                .ok_or(NavigatorError::None)?
                .iter()
                .map(|f| lex::repr(*f))
                .collect::<Vec<_>>(),
            _ => nav
                .learned_that(&atoms, &route, None)
                .ok_or(NavigatorError::None)?,
        }
    } else {
        vec![]
    };

    for line in script.lines() {
        println!("{PROMPT}{line}");
        mode.command(
            line.to_owned(),
            &mut nav,
            &mut atoms,
            &mut facets,
            &mut route,
            &mut ctx,
        )?
    }

    return Ok(());
}
