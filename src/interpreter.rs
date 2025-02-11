use crate::config::*;
use crate::is_facet;
use crate::modes::{perform_next_step, propose_next_step, Mode};
use crate::significance::Significance;
use regex::Regex;
use savan::lex;
use savan::nav::{
    errors::{NavigatorError, Result},
    facets::Facets,
    soe::Collect,
    weights::{count, Weight},
    Navigator,
};

pub trait Evaluate<T>
where
    T: Clone + PartialEq + Eq,
{
    fn command(
        &mut self,
        expr: String,
        nav: &mut Navigator,
        _nav: &mut Navigator,
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
        _nav: &mut Navigator,
        atoms: &mut Vec<String>,
        facets: &mut Vec<String>,
        route: &mut Vec<String>,
        ctx: &mut Vec<String>,
    ) -> Result<()> {
        let mut split_expr = expr.as_str().split_whitespace();

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
                    .learned_that(facets, route)
                    .ok_or(NavigatorError::None)?;
            }
            Some(IS_FACET_R) => {
                let mut k = 0;
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    for a in atoms.iter().filter(|a| re.is_match(a)) {
                        if is_facet::is_facet_r(_nav, a.to_string()) {
                            k += 2;
                            print!("{} ", a)
                        }
                    }
                } else {
                    for a in atoms.iter() {
                        if is_facet::is_facet_r(_nav, a.to_string()) {
                            k += 2;
                            print!("{} ", a)
                        }
                    }
                }
                println!("\n{k}")
            }
            Some(IS_FACET) => {
                let mut k = 0;
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    for a in atoms.iter().filter(|a| re.is_match(a)) {
                        match is_facet::is_facet(nav, a.to_string()) {
                            is_facet::State::True(a) => {
                                println!("{}", T.replace("[T]", &a))
                            }
                            is_facet::State::False(a) => {
                                println!("{}", F.replace("[F]", &a))
                            }
                            _ => {
                                k += 2;
                                println!("{}", U.replace("[U]", &a))
                            }
                        }
                    }
                } else {
                    for a in atoms.iter() {
                        match is_facet::is_facet(nav, a.to_string()) {
                            is_facet::State::True(a) => {
                                println!("{}", T.replace("[T]", &a))
                            }
                            is_facet::State::False(a) => {
                                println!("{}", F.replace("[F]", &a))
                            }
                            _ => {
                                k += 2;
                                println!("{}", U.replace("[U]", &a))
                            }
                        }
                    }
                }
                println!("\n{k}")
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                                            _nav,
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
                let y = split_expr.next().unwrap();
                if let Some(re) = split_expr.next().and_then(|s| Regex::new(r#s).ok()) {
                    nav.significance(&route, y.to_owned(), &facets, re)
                }
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
            _ => println!("noop [unknown command]"),
        }

        Ok(())
    }
}
