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

    let args = input.collect::<Vec<_>>();
    let lp = read_to_string(Path::new(&arg)).map_err(|_| NavigatorError::None)?;

    println!(
        "{} v{}\n42930d520670354cfb84ded47e54142559c70e8cd6b36d6eb2b1a24433adc78f",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    let mut nav = Navigator::new(lp, args)?;

    let mut mode = Mode::GoalOriented(None::<usize>);
    let mut route = Vec::new();
    let mut facets = nav
        .facet_inducing_atoms(route.iter())
        .ok_or(NavigatorError::None)?
        .iter()
        .map(|f| lex::repr(*f))
        .collect::<Vec<_>>();

    let mut rl = DefaultEditor::new().map_err(|_| NavigatorError::None)?;
    loop {
        match rl.readline(crate::config::PROMPT) {
            Ok(line) => {
                if let Err(err) = rl.add_history_entry(line.as_str()) {
                    eprintln!("ReadlineError: {:?}", err);
                }

                mode.command(line.replace("_", " "), &mut nav, &mut facets, &mut route)?;
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
        "{} v{}\n42930d520670354cfb84ded47e54142559c70e8cd6b36d6eb2b1a24433adc78f",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    let script =
        read_to_string(Path::new(args.last().unwrap())).map_err(|_| NavigatorError::None)?;
    args.pop();

    let mut nav = Navigator::new(lp, args)?;
    let mut mode = Mode::GoalOriented(None::<usize>);
    let mut route = Vec::new();
    let mut facets = nav
        .facet_inducing_atoms(route.iter())
        .ok_or(NavigatorError::None)?
        .iter()
        .map(|f| lex::repr(*f))
        .collect::<Vec<_>>();

    for (i, line) in script.lines().enumerate() {
        println!("{PROMPT}{line}");
        match line.starts_with("\\") {
            true => {
                let tmp = line.replace("\\", "");
                let mut src = tmp.trim().split(" | ");
                let mut pred = match src.next() {
                    Some(expr) => expr.split(" "),
                    _ => {
                        println!("error line {:?}: specify condition", i + 1);
                        std::process::exit(-1)
                    }
                };
                let inst = match src.next() {
                    Some(expr) => expr.split(".").collect::<Vec<_>>(),
                    _ => {
                        println!("error line {:?}: specify instructions in loop", i + 1);
                        std::process::exit(-1)
                    }
                };

                match pred.next() {
                    Some("_") => {
                        while !facets.is_empty() {
                            for cmd in &inst {
                                mode.command(
                                    cmd.trim().to_owned(),
                                    &mut nav,
                                    &mut facets,
                                    &mut route,
                                )?
                            }
                        }
                    }
                    Some("!=") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() != x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() != x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        _ => {
                            println!("unknown lhs specified in line {}", i + 1);
                            std::process::exit(-1)
                        }
                    },
                    Some(">") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() > x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() > x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        _ => {
                            println!("unknown lhs specified in line {}", i + 1);
                            std::process::exit(-1)
                        }
                    },
                    Some(">=") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() >= x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() >= x {
                                    for cmd in &inst {
                                        mode.command(
                                            cmd.trim().to_owned(),
                                            &mut nav,
                                            &mut facets,
                                            &mut route,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("unknown rhs specified in line {}", i + 1);
                                std::process::exit(-1)
                            }
                        },
                        _ => {
                            println!("unknown lhs specified in line {}", i + 1);
                            std::process::exit(-1)
                        }
                    },
                    _ => {
                        println!("invalid syntax line {:?}: provide loop", i + 1);
                        std::process::exit(-1)
                    }
                };
            }
            _ => mode.command(line.to_owned(), &mut nav, &mut facets, &mut route)?,
        };
    }

    return Ok(());
}
