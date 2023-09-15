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
            println!("Expected input.");
            return Ok(());
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
            println!("Expected input.");
            return Ok(());
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
        println!("{line}");
        match line.starts_with("\\") {
            true => {
                let tmp = line.replace("\\", "");
                let mut src = tmp.trim().split(" | ");
                let pred = match src.next() {
                    Some("#f!0") => |x: &Vec<String>| !x.is_empty(),
                    Some("#f<") => |x: &Vec<String>| !x.is_empty(),
                    Some(expr) => {
                        if expr.contains("==") {
                            let mut xs = expr.split("==");
                            match xs.next() {
                                Some("#f!0") => {
                                    match xs.next().and_then(|n| n.parse::<usize>().ok()) {
                                        Some(n) => unimplemented!(),
                                        _ => unimplemented!(),
                                    }
                                }
                                _ => {
                                    println!(
                                        "invalid syntax line {:?}: unsupported condition",
                                        i + 1
                                    );
                                    std::process::exit(-1)
                                }
                            }
                        } else {
                            println!("invalid syntax line {:?}: unsupported condition", i + 1);
                            std::process::exit(-1)
                        }
                    }
                    _ => {
                        println!(
                            "invalid syntax line {:?}: provide loop breaking condition",
                            i + 1
                        );
                        std::process::exit(-1)
                    }
                };
                let loopp = match src.next() {
                    Some(expr) => {
                        while pred(&facets) {
                            for cmd in expr.split(".") {
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
                        println!("invalid syntax line {:?}: provide loop", i + 1);
                        std::process::exit(-1)
                    }
                };
            }
            _ => mode.command(line.replace("_", " "), &mut nav, &mut facets, &mut route)?,
        };
    }
    std::process::exit(0);

    //let pre = script
    //    .lines()
    //    .filter(|l| !l.is_empty())
    //    .take_while(|s| *s != "{")
    //    .map(|l| l.chars().take_while(|c| *c != '-').collect::<String>())
    //    .collect::<Vec<_>>();
    //let src = script
    //    .lines()
    //    .filter(|l| !l.is_empty())
    //    .skip_while(|s| *s != "{")
    //    .skip(1)
    //    .take_while(|s| *s != "}")
    //    .map(|l| l.chars().take_while(|c| *c != '-').collect::<String>())
    //    .collect::<Vec<_>>();
    //let end = script
    //    .lines()
    //    .filter(|l| !l.is_empty())
    //    .skip_while(|s| *s != "}")
    //    .skip(1)
    //    .map(|l| l.chars().take_while(|c| *c != '-').collect::<String>())
    //    .collect::<Vec<_>>();

    //for line in &pre {
    //    println!("{PROMPT} {}", &line);
    //    mode.command(line.replace("_", " "), &mut nav, &mut facets, &mut route)?;
    //    if facets.is_empty() {
    //        break;
    //    }
    //}
    //'runtime: loop {
    //    for line in &src {
    //        println!("{PROMPT} {}", &line);
    //        if facets.is_empty() {
    //            break 'runtime;
    //        }
    //    }
    //}
    //assert!(facets.is_empty());
    //for line in &end {
    //    println!("{PROMPT} {}", &line);
    //    mode.command(line.replace("_", " "), &mut nav, &mut facets, &mut route)?;
    //}

    return Ok(());
}
