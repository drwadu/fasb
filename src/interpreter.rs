use crate::config::*;
use crate::modes::{perform_next_step, propose_next_step, Mode};
use regex::Regex;
use savan::lex;
use savan::nav::{
    errors::{NavigatorError, Result},
    facets::Facets,
    weights::{count, Weight},
    Navigator,
};

#[cfg(feature = "verbose")]
use std::time::Instant;

pub trait Evaluate<T>
where
    T: Clone + PartialEq + Eq,
{
    fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
    ) -> Result<()>;
}
impl Evaluate<Option<usize>> for Mode<Option<usize>> {
    fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
    ) -> Result<()> {
        let mut expr = expr.split(" ");
        match expr.next() {
            Some(ACTIVATE_FACETS) => {
                #[cfg(feature = "verbose")]
                println!("% activation started");
                #[cfg(feature = "verbose")]
                let start = Instant::now();

                expr.for_each(|f| {
                    route.push(f.to_owned());
                });
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();

                #[cfg(feature = "verbose")]
                println!("% activation elapsed: {:?}", start.elapsed());
            }
            Some(ENUMERATE_SOLUTIONS) => {
                #[cfg(feature = "verbose")]
                println!("% enumeration started");
                #[cfg(feature = "verbose")]
                let start = Instant::now();

                let n = nav.enumerate_solutions(
                    expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    route.iter().map(|s| s.as_ref()).chain(expr),
                )?;
                println!("found {:?}", n);

                #[cfg(feature = "verbose")]
                println!("% enumeration elapsed: {:?}", start.elapsed());
            }
            Some(SHOW_FACETS) => {
                if let Some(re) = expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    facets
                        .iter()
                        .filter(|f| re.is_match(f))
                        .for_each(|f| print!("{} ", f));
                } else {
                    facets.iter().for_each(|f| print!("{} ", f));
                }
                println!()
            }
            Some(FACET_COUNT) => {
                println!("{:?}", 2 * facets.len())
            }
            Some(FACET_COUNTS) => {
                let ovr_count = match self {
                    Self::MaxWeightedFacetCounting(Some(c)) => *c,
                    Self::MinWeightedFacetCounting(Some(c)) => *c,
                    _ => 2 * facets.len(),
                } as f32;
                let mut weight = Weight::FacetCounting;

                if let Some(re) = expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    for f in facets.iter().filter(|f| re.is_match(f)) {
                        route.push(f.to_owned());
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                } else {
                    for f in facets.iter() {
                        route.push(f.to_owned());
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                }
            }
            Some(ANSWER_SET_COUNT) => {
                let n = nav.enumerate_solutions_quietly(
                    expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    route.iter().map(|s| s.as_ref()).chain(expr),
                )?;
                println!("{:?}", n)
            }
            Some(ANSWER_SET_COUNTS) => {
                let mut weight = Weight::AnswerSetCounting;
                let ovr_count = match self {
                    Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                    Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                    _ => count(&mut weight, nav, route.iter()).ok_or(NavigatorError::None)?,
                } as f32;

                if let Some(re) = expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    for f in facets.iter().filter(|f| re.is_match(f)) {
                        route.push(f.to_owned());
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                } else {
                    for f in facets.iter() {
                        route.push(f.to_owned());
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                }
            }
            Some(SHOW_ROUTE) => {
                route.iter().for_each(|f| print!("{f} "));
                println!();
            }
            Some(DEL_LAST) => {
                route.pop();
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            Some(CLEAR_ROUTE) => {
                route.clear();
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            Some(CHANGE_MODE) => match expr.next() {
                Some("min#f") => {
                    *self = Mode::MinWeightedFacetCounting(
                        expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    )
                }
                Some("max#f") => {
                    *self = Mode::MaxWeightedFacetCounting(
                        expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    )
                }
                Some("min#a") => {
                    *self = Mode::MinWeightedAnswerSetCounting(
                        expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    )
                }
                Some("max#a") => {
                    *self = Mode::MaxWeightedAnswerSetCounting(
                        expr.next().and_then(|n| n.parse::<usize>().ok()).take(),
                    )
                }
                Some("go") => {
                    *self =
                        Mode::GoalOriented(expr.next().and_then(|n| n.parse::<usize>().ok()).take())
                }
                _ => println!("specify mode among {{{{min,max}}#{{f,a,s}}, go}}"),
            },

            Some(PROPOSE_STEP) => {
                let fs = if let Some(re) = expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    facets
                        .iter()
                        .filter(|f| re.is_match(f))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    facets.to_vec()
                };
                match propose_next_step(self, nav, route, &fs) {
                    Some((f, Some(c))) => println!("{f} {:?}", c),
                    Some((f, None)) => println!("{f} _"),
                    _ => println!("noop"),
                }
            }
            Some(TAKE_STEP) => {
                let fs = if let Some(re) = expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    facets
                        .iter()
                        .filter(|f| re.is_match(f))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    facets.to_vec()
                };
                #[cfg(feature = "verbose")]
                println!("% performing step started");
                #[cfg(feature = "verbose")]
                let start = Instant::now();

                let ovr_count = match self {
                    Self::MaxWeightedFacetCounting(Some(c)) => *c,
                    Self::MinWeightedFacetCounting(Some(c)) => *c,
                    Self::MaxWeightedFacetCounting(None) | Self::MinWeightedFacetCounting(None) => {
                        2 * facets.len()
                    }
                    Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                    Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                    Self::MaxWeightedAnswerSetCounting(None)
                    | Self::MinWeightedAnswerSetCounting(None) => {
                        count(&mut Weight::AnswerSetCounting, nav, route.iter())
                            .ok_or(NavigatorError::None)?
                    }
                    Self::GoalOriented(_) => usize::default(),
                } as f32;

                match perform_next_step(self, nav, route, &fs) {
                    Some((f, Some(c))) => {
                        println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c);
                        self.update(Some(c));
                        *facets = nav
                            .facet_inducing_atoms(route.iter())
                            .ok_or(NavigatorError::None)?
                            .iter()
                            .map(|f| lex::repr(*f))
                            .collect();
                    }
                    Some((f, None)) => {
                        println!("{f} _");
                        *facets = nav
                            .facet_inducing_atoms(route.iter())
                            .ok_or(NavigatorError::None)?
                            .iter()
                            .map(|f| lex::repr(*f))
                            .collect();
                    }
                    _ => println!("noop"),
                }

                #[cfg(feature = "verbose")]
                eprintln!("% performing step elapsed: {:?}", start.elapsed());
            }
            Some(QUIT) => std::process::exit(0),
            Some("man") => crate::config::manual(),
            _ => println!("noop [unknown command]"),
        }

        Ok(())
    }
}
