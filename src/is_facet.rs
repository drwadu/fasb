use savan::nav::Navigator;

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

pub fn copy_atom(atom: String) -> String {
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

pub fn is_facet_r(nav: &mut Navigator, atom: String) -> bool {
    let c = copy_atom(atom.clone());
    match nav.enumerate_solutions_quietly(Some(1), [atom.clone(), format!("~{c}")].iter()) {
        Ok(1) => true,
        _ => false,
    }
}

#[derive(PartialEq, Eq)]
pub enum State {
    True(String),
    Uncertain(String),
    False(String),
}

pub fn is_facet(nav: &mut Navigator, atom: String) -> State {
    match nav.enumerate_solutions_quietly(Some(1), [atom.clone()].iter()) {
        Ok(1) => match nav.enumerate_solutions_quietly(Some(1), [format!("~{atom}")].iter()) {
            Ok(1) => State::Uncertain(atom),
            _ => State::True(atom),
        },
        _ => State::False(atom),
    }
}
