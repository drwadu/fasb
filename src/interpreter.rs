use crate::config::*;
use crate::is_facet;
use crate::modes::{perform_next_step, propose_next_step, Mode};
use crate::significance::Significance;
use crate::wfc::parse_weighted_facets_from_file;
use crate::wfc::weighted_facet_count;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use regex::Regex;
use savan::lex;
use savan::nav::{
    errors::{NavigatorError, Result},
    facets::Facets,
    soe::Collect,
    weights::{count, count_projecting, Weight},
    Navigator,
};
use std::fmt::Write;
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub trait Evaluate<T>
where
    T: Clone + PartialEq + Eq,
{
    fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()>;
}
impl Evaluate<Option<usize>> for Mode<Option<usize>> {
    fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()> {
        let e = expr.clone();
        let mut split_expr = e.as_str().split_whitespace();

        match split_expr.next() {
            Some(ACTIVATE_FACETS) => {
                split_expr.for_each(|f| {
                    route.push(f.to_owned());
                });
                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            Some(ACTIVATE_FACETS_LT) => {
                split_expr.for_each(|f| {
                    route.push(f.to_owned());
                });
                *facets = nav
                    .learned_that(facets, route, None)
                    .ok_or(NavigatorError::None)?;
            }
            Some(ACTIVATE_FACETS_LAZY) => {
                split_expr.for_each(|f| {
                    route.push(f.to_owned());
                });
            }
            Some(COMPUTE_FACETS) => {
                let start = Instant::now();
                *facets = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    nav.facet_inducing_atoms(route.iter())
                        .ok_or(NavigatorError::None)?
                        .iter()
                        .map(|f| lex::repr(*f))
                        .filter(|a| re.is_match(&a))
                        .collect::<Vec<_>>()
                } else {
                    nav.facet_inducing_atoms(route.iter())
                        .ok_or(NavigatorError::None)?
                        .iter()
                        .map(|f| lex::repr(*f))
                        .collect()
                };
                println!("time elapsed: {:?}", start.elapsed())
            }
            Some(ENTAILMENT) => {
                let start = Instant::now();
                let fst = split_expr.next();

                match fst {
                    Some("%") => {
                        if let Some(xs) = nav
                            .cautious_consequences(route.iter())
                            .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                        {
                            if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                                atoms.iter().filter(|f| re.is_match(f)).for_each(|f| {
                                    if xs.contains(f) {
                                        println!("{f}")
                                    }
                                });
                            } else {
                                atoms.iter().for_each(|f| {
                                    if xs.contains(f) {
                                        println!("{f}")
                                    }
                                });
                            }
                        }
                    }
                    Some("%%") => {
                        if let Some(xs) = nav
                            .brave_consequences(route.iter())
                            .map(|fs| fs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>())
                        {
                            if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                                atoms.iter().filter(|f| re.is_match(f)).for_each(|f| {
                                    if !xs.contains(f) {
                                        println!("{f}")
                                    }
                                });
                            } else {
                                atoms.iter().for_each(|f| {
                                    if !xs.contains(f) {
                                        println!("{f}")
                                    }
                                });
                            }
                        }
                    }
                    Some(&_) | None => {
                        if let Some(bcs) = nav.brave_consequences(route.iter()) {
                            if bcs.is_empty() {
                                println!("no answer set")
                            } else {
                                // NOTE: is option
                                //let initial_facets = nav.facet_inducing_atoms(std::iter::empty())
                                //.ok_or(NavigatorError::None)?
                                //.iter()
                                //.map(|f| lex::repr(*f))
                                //.collect();
                                let bcs_str = bcs.iter().map(|f| lex::repr(*f)).collect::<Vec<_>>();
                                if let Some(re) = fst.and_then(|s| Regex::new(r#s).ok()) {
                                    atoms.iter().filter(|f| re.is_match(f)).for_each(|f| {
                                        if !bcs_str.contains(f) {
                                            println!("\x1b[0;30;41m{}\x1b[0m", f)
                                        } else {
                                            if let Ok(1) = nav.enumerate_solutions_quietly(
                                                Some(1),
                                                route.iter().chain([format!("~{f}")].iter()),
                                            ) {
                                            } else {
                                                println!("\x1b[0;30;42m{}\x1b[0m", f)
                                            }
                                        }
                                    });
                                } else {
                                    atoms.iter().for_each(|f| {
                                        if !bcs_str.contains(f) {
                                            println!("\x1b[0;30;41m{}\x1b[0m", f)
                                        } else {
                                            if let Ok(1) = nav.enumerate_solutions_quietly(
                                                Some(1),
                                                route.iter().chain([format!("~{f}")].iter()),
                                            ) {
                                            } else {
                                                println!("\x1b[0;30;42m{}\x1b[0m", f)
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                println!("ent time elapsed: {:?}", start.elapsed())
            }
            Some(COMPUTE_FACETS_SU) => {
                let xs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms
                        .iter()
                        .filter(|a| re.is_match(a))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    atoms.iter().cloned().collect::<Vec<_>>()
                };

                let mut or = ":-".to_owned();
                xs.iter().for_each(|a| {
                    or = format!("{or} not {a},");
                });
                or = format!("{}.", &or[..or.len() - 1]);

                let shows = nav
                    .symbols()
                    .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
                    .map(|(s, n)| format!("#show {s}/{n}."))
                    .collect::<Vec<_>>()
                    .join("\n");

                let s = format!("{shows}\n{or}");

                nav.add_rule(s.clone())?;

                *facets = nav
                    .facet_inducing_atoms_projecting(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();

                nav.remove_rule(s)?;
            }
            Some("!?soe") => {
                let xs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms
                        .iter()
                        .filter(|a| re.is_match(a))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    atoms.iter().cloned().collect::<Vec<_>>()
                };
                let shows = nav
                    .symbols()
                    .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
                    .map(|(s, n)| format!("#show {s}/{n}."))
                    .collect::<Vec<_>>()
                    .join("\n");
                nav.add_rule(shows.clone()).unwrap();
                let cc = nav.cautious_consequences_projecting(route.iter());
                nav.remove_rule(shows).unwrap();

                let ys = cc
                    .map(|cc| {
                        let cc_ = cc.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                        xs.iter()
                            .filter(move |x| !cc_.contains(x))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap();
                let shows = nav
                    .symbols()
                    .filter(|(s, _)| ys.iter().any(|a| a.starts_with(s)))
                    .map(|(s, n)| format!("#show {s}/{n}."))
                    .collect::<Vec<_>>()
                    .join("\n");
                nav.add_rule(shows.clone()).unwrap();
                nav.add_arg("--project=show")?;

                *facets = nav.sieve_quiet(&ys).unwrap();

                nav.remove_rule(shows).unwrap();
            }
            Some(IS_FACET_R) => {
                let mut fs = vec![];
                let mut k = 0;
                let xs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms.iter().filter(|a| re.is_match(a)).collect::<Vec<_>>()
                } else {
                    atoms.iter().collect::<Vec<_>>()
                };
                let (n, mut m) = (atoms.len() as u64, 0);
                let pb = ProgressBar::new(n);
                let style = "{spinner:.green} [{elapsed_precise}] [{wide_bar}] ({eta})";
                pb.set_style(ProgressStyle::with_template(style).unwrap().with_key(
                    "eta",
                    |state: &ProgressState, w: &mut dyn Write| {
                        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    },
                ));

                let lp = nav.program();
                let clp = is_facet::copy_program(lp.clone());
                nav.add_rule(clp.clone())?;

                for x in xs {
                    if is_facet::is_facet_r(nav, x.to_string()) {
                        fs.push(x.to_owned());
                        k += 2;
                    }
                    m += 1;
                    pb.set_position(m);
                    thread::sleep(Duration::from_millis(12));
                }
                pb.finish_with_message("computed facets");
                println!("\n{k}");
                *facets = fs;

                nav.remove_rule(clp)?;
            }
            Some(IS_FACET) => {
                if let Some(x) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    println!("{:?}", is_facet::is_facet(nav, x.to_string()))
                }
            }
            Some(WEIGHTED_FACET_COUNT) => {
                match split_expr
                    .next()
                    .and_then(|filename| parse_weighted_facets_from_file(filename))
                    .and_then(|wfcs| weighted_facet_count(nav, route.to_vec(), wfcs))
                {
                    Some(score) => println!("{:?}", score),
                    _ => println!("NA"),
                }
            }
            Some(WEIGHTED_FACET_COUNTS) => {
                match split_expr
                    .next()
                    .and_then(|filename| parse_weighted_facets_from_file(filename))
                {
                    Some(wfcs) => {
                        if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                            for f in facets.iter().filter(|f| re.is_match(f)) {
                                route.push(f.to_owned());
                                match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                                    Some(score) => println!("{:?} {f}", score),
                                    _ => println!("NA"),
                                }
                                route.pop();
                                route.push(format!("~{f}"));
                                match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                                    Some(score) => println!("{:?} ~{f}", score),
                                    _ => println!("NA"),
                                }
                                route.pop();
                            }
                        } else {
                            for f in facets.iter() {
                                route.push(f.to_owned());
                                match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                                    Some(score) => println!("{:?} {f}", score),
                                    _ => println!("NA"),
                                }
                                route.pop();
                                route.push(format!("~{f}"));
                                match weighted_facet_count(nav, route.to_vec(), wfcs.clone()) {
                                    Some(score) => println!("{:?} ~{f}", score),
                                    _ => println!("NA"),
                                }
                                route.pop();
                            }
                        }
                    }
                    _ => println!("NA"),
                }
            }
            Some(ENUMERATE_SOLUTIONS) => {
                let n = nav.enumerate_solutions(
                    split_expr
                        .next()
                        .and_then(|n| n.parse::<usize>().ok())
                        .take(),
                    route.iter().map(|s| s.as_ref()).chain(split_expr),
                )?;
                println!("found {:?}", n);
            }
            Some(SHOW_FACETS) => {
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
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

                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
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
            Some(FACET_COUNTS_PROJECTING) => {
                let ovr_count = match self {
                    Self::MaxWeightedFacetCounting(Some(c)) => *c,
                    Self::MinWeightedFacetCounting(Some(c)) => *c,
                    _ => 2 * facets.len(),
                } as f32;
                let mut weight = Weight::FacetCounting;

                let xs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms
                        .iter()
                        .filter(|a| re.is_match(a))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    atoms.iter().cloned().collect::<Vec<_>>()
                };

                let mut or = ":-".to_owned();
                xs.iter().for_each(|a| {
                    or = format!("{or} not {a},");
                });
                or = format!("{}.", &or[..or.len() - 1]);

                let shows = nav
                    .symbols()
                    .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
                    .map(|(s, n)| format!("#show {s}/{n}."))
                    .collect::<Vec<_>>()
                    .join("\n");

                let s = format!("{shows}\n{or}");

                nav.add_rule(s.clone())?;

                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    for f in facets.iter().filter(|f| re.is_match(f)) {
                        route.push(f.to_owned());
                        count_projecting(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", c, 1.0 - (c as f32 / ovr_count)))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count_projecting(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", c, 1.0 - (c as f32 / ovr_count)))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                } else {
                    for f in facets.iter() {
                        route.push(f.to_owned());
                        count_projecting(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} {f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                        route.push(format!("~{f}"));
                        count_projecting(&mut weight, nav, route.iter())
                            .map(|c| println!("{:.4} {:?} ~{f}", 1.0 - (c as f32 / ovr_count), c))
                            .ok_or(NavigatorError::None)?;
                        route.pop();
                    }
                }

                nav.remove_rule(s)?;
            }
            Some(ANSWER_SET_COUNT) => {
                let n = nav.enumerate_solutions_quietly(
                    split_expr
                        .next()
                        .and_then(|n| n.parse::<usize>().ok())
                        .take(),
                    route.iter().map(|s| s.as_ref()).chain(split_expr),
                )?;
                println!("{:?}", n)
            }
            Some(ANSWER_SET_COUNTS) => {
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    let mut weight = Weight::AnswerSetCounting;
                    let ovr_count = match self {
                        Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                        Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                        _ => count(&mut weight, nav, route.iter()).ok_or(NavigatorError::None)?,
                    } as f32;
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
                    let mut weight = Weight::AnswerSetCounting;
                    let ovr_count = match self {
                        Self::MaxWeightedAnswerSetCounting(Some(c)) => *c,
                        Self::MinWeightedAnswerSetCounting(Some(c)) => *c,
                        _ => count(&mut weight, nav, route.iter()).ok_or(NavigatorError::None)?,
                    } as f32;
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
                if !ctx.is_empty() {
                    ctx.first().map(|f| println!("{f}"));
                }

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
            Some(DISPLAY_MODE) => println!("{}", self),
            Some(CHANGE_MODE) => match split_expr.next() {
                Some("min#f") => {
                    *self = Mode::MinWeightedFacetCounting(
                        split_expr
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .take(),
                    )
                }
                Some("max#f") => {
                    *self = Mode::MaxWeightedFacetCounting(
                        split_expr
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .take(),
                    )
                }
                Some("min#a") => {
                    *self = Mode::MinWeightedAnswerSetCounting(
                        split_expr
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .take(),
                    )
                }
                Some("max#a") => {
                    *self = Mode::MaxWeightedAnswerSetCounting(
                        split_expr
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .take(),
                    )
                }
                Some("go") => {
                    *self = Mode::GoalOriented(
                        split_expr
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .take(),
                    )
                }
                _ => println!("error: specify mode among {{{{min,max}}#{{f,a,s}}, go}}"),
            },
            Some(PROPOSE_STEP) => {
                let fs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
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
                let start = Instant::now();
                let fs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    facets
                        .iter()
                        .filter(|f| re.is_match(f))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    facets.to_vec()
                };
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
                        println!("_ _ {f}");
                        *facets = nav
                            .facet_inducing_atoms(route.iter())
                            .ok_or(NavigatorError::None)?
                            .iter()
                            .map(|f| lex::repr(*f))
                            .collect();
                    }
                    _ => println!("noop"),
                }

                println!("tak time elapsed: {:?}", start.elapsed())
            }
            Some(QUIT) => std::process::exit(0),
            Some("man") => crate::config::manual(),
            Some("\\") => {
                let tmp = expr.replace("\\", "");

                let mut src = tmp.trim().split(" | ");
                let mut pred = match src.next() {
                    Some(expr) => expr.split(" "),
                    _ => {
                        println!("error: specify condition");
                        return Ok(());
                    }
                };
                let inst = match src.next() {
                    Some(expr) => expr.split(".").collect::<Vec<_>>(),
                    _ => {
                        println!("error: found no instructions");
                        return Ok(());
                    }
                };

                match pred.next() {
                    Some("!=") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() != x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() != x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("error: unknown lhs");
                            return Ok(());
                        }
                    },
                    Some(">") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() > x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() > x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("error: unknown lhs");
                            return Ok(());
                        }
                    },
                    Some(">=") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() >= x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() >= x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("error: unknown lhs");
                            return Ok(());
                        }
                    },
                    Some("<") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() < x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() < x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("error: unknown lhs");
                            return Ok(());
                        }
                    },
                    Some("<=") => match pred.next() {
                        Some("#f") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && 2 * facets.len() <= x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        Some("#r") => match pred.next().and_then(|n| n.parse::<usize>().ok()) {
                            Some(x) => {
                                while !facets.is_empty() && route.len() <= x {
                                    for cmd in &inst {
                                        self.command(
                                            cmd.trim().to_owned(),
                                            nav,
                                            atoms,
                                            facets,
                                            route,
                                            ctx,
                                        )?
                                    }
                                }
                            }
                            _ => {
                                println!("error: unknown rhs");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("error: unknown lhs");
                            return Ok(());
                        }
                    },
                    _ => {
                        println!("error: provide instructions");
                        return Ok(());
                    }
                };
            }
            Some(IS_ATOM) => match split_expr.next().and_then(|a| nav.is_known(a.to_owned())) {
                Some(v) => println!("{v}"),
                _ => println!("error: invalid atom"),
            },
            Some(SHOW_ATOMS) => {
                nav.atoms().for_each(|a| {
                    print!("{a} ");
                });
                println!();
            }
            Some(FILTER_ATOMS) => {
                let mut k = 0;
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms.iter().filter(|f| re.is_match(f)).for_each(|f| {
                        k += 1;
                        print!("{} ", f)
                    });
                } else {
                    atoms.iter().for_each(|f| {
                        k += 1;
                        print!("{} ", f)
                    });
                }
                println!("\n{k}")
            }
            Some(SHOW_PROGRAM) => {
                println!("{}", nav.program());
            }
            Some(SOE) => {
                let fs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    facets
                        .iter()
                        .filter(|f| re.is_match(f))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    facets.to_vec()
                };
                nav.sieve(&fs)?;
            }
            Some(CONTEXT) => {
                ctx.into_iter()
                    .skip(1)
                    .for_each(|r| unsafe { nav.remove_rule(r).unwrap_unchecked() });

                ctx.clear();

                match split_expr.next() {
                    Some(cnf) => {
                        ctx.push(cnf.to_string());

                        let clauses = cnf.split("&");
                        for clause in clauses {
                            let body = clause
                                .split("|")
                                .map(|lit| match lit.starts_with('~') {
                                    true => lit[1..].to_owned(),
                                    _ => format!("not {lit}"),
                                })
                                .collect::<Vec<_>>()
                                .join(",");

                            let ic = format!(":- {body}. ");

                            ctx.push(ic.clone());

                            nav.add_rule(ic)?;
                        }
                    }
                    _ => (),
                };

                *facets = nav
                    .facet_inducing_atoms(route.iter())
                    .ok_or(NavigatorError::None)?
                    .iter()
                    .map(|f| lex::repr(*f))
                    .collect();
            }
            Some(SIGNIFICANCE) => {
                let start = Instant::now();
                let y = split_expr.next().unwrap();
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    nav.significance(&route, y.to_owned(), &facets, re)
                }
                println!("sig time elapsed: {:?}", start.elapsed())
            }
            Some(SIGNIFICANCE_PROJECTING) => {
                let y = split_expr.next().unwrap();

                let xs = if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    atoms
                        .iter()
                        .filter(|a| re.is_match(a))
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    atoms.iter().cloned().collect::<Vec<_>>()
                };

                let mut or = ":-".to_owned();
                xs.iter().for_each(|a| {
                    or = format!("{or} not {a},");
                });
                or = format!("{}.", &or[..or.len() - 1]);

                let shows = nav
                    .symbols()
                    .filter(|(s, _)| xs.iter().any(|a| a.starts_with(s)))
                    .map(|(s, n)| format!("#show {s}/{n}."))
                    .collect::<Vec<_>>()
                    .join("\n");

                let s = format!("{shows}\n{or}");

                nav.add_rule(s.clone())?;

                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    nav.significance_projecting(&route, y.to_owned(), &facets, re)
                }

                nav.remove_rule(s.clone())?;
            }
            Some(ENUMERATE_PROJECTED_SOLUTIONS) => {
                let n = nav.enumerate_projected_solutions(
                    split_expr
                        .next()
                        .and_then(|n| n.parse::<usize>().ok())
                        .take(),
                    route.iter().map(|s| s.as_ref()).chain(split_expr),
                    facets.clone(),
                )?;
                println!("found {:?}", n);
            }
            Some(cmd) => {
                if cmd.starts_with("//") {
                    return Ok(());
                }
                println!("noop [unknown command]");
            }
            _ => eprintln!("unknown error"),
        }

        Ok(())
    }
}
