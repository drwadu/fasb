use std::collections::HashSet;

use indicatif::ProgressBar;
use savan::nav::Navigator;

use std::thread;
use std::time::Duration;

pub fn copy_program(lp: String) -> String {
    let mut xs = vec![];
    for x in lp.chars() {
        match x {
            ' ' | ',' | '.' | ';' | '(' | ')' | ':' | '-' | '+' | '\\' | '/' | '{' | '}' | '\n'
            | '\t' | '"' | '>' | '<' | '=' | '#' | '%' => xs.push(x),
            _ => {
                if x.is_numeric() {
                    xs.push(x);
                    continue;
                }
                xs.push('_');
                xs.push(x)
            }
        }
    }

    xs.iter()
        .collect::<String>()
        .replace("_n_o_t", "not")
        .replace("#_c_o_n_s_t", "#const")
        .replace("#_s_h_o_w", "#show")
        .replace("#_p_r_o_j_e_c_t", "#project")
        .replace("#_c_o_u_n_t", "#count")
        .replace("#_s_u_m", "#sum")
        .to_string()
}

pub fn copy_atom(atom: &str) -> String {
    let mut xs = vec![];
    for x in atom.chars() {
        match x {
            ' ' | ',' | '.' | ';' | '(' | ')' | ':' | '-' | '+' | '\\' | '/' | '{' | '}' | '\n'
            | '\t' | '"' | '>' | '<' | '=' | '#' | '%' => xs.push(x),
            _ => {
                if x.is_numeric() {
                    xs.push(x);
                    continue;
                }
                xs.push('_');
                xs.push(x)
            }
        }
    }

    xs.iter().collect::<String>()
}

pub fn copied_atom(atom: &str) -> String {
    let mut xs = vec![];
    for (i, x) in atom.chars().enumerate() {
        match x {
            '_' => {
                if (i + 1) % 2 != 0 {
                    continue;
                }
            }
            _ => xs.push(x),
        }
    }

    xs.iter().collect::<String>()
}

pub fn is_facet_r(nav: &mut Navigator, atom: String) -> bool {
    let c = copy_atom(&atom);
    match nav.enumerate_solutions_quietly(Some(1), [atom.clone(), format!("~{c}")].iter()) {
        Ok(1) => true,
        _ => false,
    }
}

pub fn is_facet(nav: &mut Navigator, atom: String) -> bool {
    match nav.enumerate_solutions_quietly(Some(1), [atom.clone()].iter()) {
        Ok(1) => match nav.enumerate_solutions_quietly(Some(1), [format!("~{atom}")].iter()) {
            Ok(1) => true,
            _ => false,
        },
        _ => false,
    }
}

#[allow(unused)]
pub fn rec_red(
    nav: &mut Navigator,
    target_atoms: Vec<String>,
    mut fs: Vec<String>,
    mut seen: Vec<String>,
    n: u64,
    mut k: u64,
    pb: ProgressBar,
) -> Vec<String> {
    //dbg!("new", &target_atoms, &fs, &seen, n, k);
    if k >= n {
        return fs;
    }
    if let Some(alpha) = target_atoms.iter().filter(|a| !seen.contains(a)).next() {
        //dbg!(k, &rest);
        if k >= n {
            return fs;
        }
        let mut rest = vec![];
        seen.push(alpha.to_string());
        let delta = vec![alpha.clone(), format!("~{}", copy_atom(&alpha))];
        match nav.one_or_none(delta.iter()) {
            Some(answer_set) => {
                fs.push(alpha.to_string());
                //println!("proof {alpha} {:?} {k} check", &target_atoms);
                k += 1;
                pb.set_position(k);
                thread::sleep(Duration::from_millis(12));
                if k >= n {
                    return fs;
                }
                //dbg!(&answer_set);
                for beta in answer_set.iter() {
                    if beta == alpha || seen.contains(beta) || seen.contains(&copied_atom(beta)) {
                        continue;
                    }
                    match beta.starts_with('_') {
                        true => match !answer_set.contains(&copied_atom(beta)) {
                            true => {
                                //println!("proof {beta} {:?} {k} nocopy", &target_atoms);
                                //dbg!(beta, copied_atom(beta));
                                seen.push(copied_atom(beta));
                                fs.push(copied_atom(beta));
                                k += 1;
                                pb.set_position(k);
                                thread::sleep(Duration::from_millis(12));
                                if k >= n {
                                    return fs;
                                }
                            }
                            _ => {
                                if !seen.contains(&copied_atom(beta)) {
                                    rest.push(copied_atom(beta));
                                }
                            }
                        },
                        _ => match !answer_set.contains(&copy_atom(beta)) {
                            true => {
                                //println!("proof {beta} {:?} {k} noactual", &target_atoms);
                                seen.push(beta.to_string());
                                fs.push(beta.to_string());
                                k += 1;
                                pb.set_position(k);
                                thread::sleep(Duration::from_millis(12));
                                if k >= n {
                                    return fs;
                                }
                            }
                            _ => {
                                if !seen.contains(&copied_atom(beta)) {
                                    rest.push(copied_atom(beta));
                                }
                            }
                        },
                    }
                }

                return rec_red(
                    nav,
                    rest.iter()
                        .cloned()
                        .chain(target_atoms.iter().cloned().filter(|a| !seen.contains(a)))
                        .collect::<HashSet<_>>()
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>(),
                    fs,
                    seen.clone(),
                    n,
                    k,
                    pb,
                );
            }
            _ => {
                //println!("disproof {alpha} {:?} {k}", &target_atoms);
                k += 1;
                pb.set_position(k);
                thread::sleep(Duration::from_millis(12));
                return rec_red(
                    nav,
                    target_atoms.iter().cloned().skip(1).collect::<Vec<_>>(),
                    fs,
                    seen.clone(),
                    n,
                    k,
                    pb,
                );
            }
        }
    } else {
        return fs;
    }

    //fs.iter()
    //    .cloned()
    //    .collect::<HashSet<_>>()
    //    .iter()
    //    .cloned()
    //    .collect::<Vec<_>>()
}

#[allow(unused)]
pub fn rec_soe(
    nav: &mut Navigator,
    target_atoms: Vec<String>,
    mut fs: Vec<String>,
    n: u64,
    mut k: u64,
    pb: ProgressBar,
) -> Vec<String> {
    //if let Some((mut to_observe, fs_, falsified_atom)) = nav.sieve_quiet(&target_atoms) {
    //    for f in fs_.iter() {
    //        fs.push(f.to_string())
    //    }
    //    to_observe
    //        .iter()
    //        .position(|x| *x == falsified_atom)
    //        .map(|i| to_observe.remove(i))
    //        .unwrap();
    //    println!("some {:?} {:?} {:?}", to_observe.len(), fs_.len(), fs.len());
    //    rec_soe(nav, to_observe, fs.clone(), n, k, pb)
    //} else {
    //    println!("done {:?}", target_atoms.clone());
    //    for f in target_atoms {
    //        fs.push(f)
    //    }
    //    return fs;
    //}
    unimplemented!()
}
