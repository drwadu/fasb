use savan::nav::{
    weights::{count, Weight},
    Navigator,
};
use std::fmt;

pub fn propose_next_step<T>(
    mode: &mut impl Step<T>,
    nav: &mut Navigator,
    active: &mut Vec<String>,
    facets: &[String],
) -> Option<(String, T)>
where
    T: Clone + Eq + PartialEq,
{
    mode.propose_facet(nav, active, facets)
}

pub fn perform_next_step<T>(
    mode: &mut impl Step<T>,
    nav: &mut Navigator,
    active: &mut Vec<String>,
    facets: &[String],
) -> Option<(String, T)>
where
    T: Clone + Eq + PartialEq,
{
    mode.propose_facet(nav, active, facets)
        .and_then(|(facet, count)| {
            active.push(facet.clone());
            Some((facet, count))
        })
}

#[derive(Clone)]
pub enum Mode<T> {
    GoalOriented(T),
    MinWeightedFacetCounting(T),
    MaxWeightedFacetCounting(T),
    MinWeightedAnswerSetCounting(T),
    MaxWeightedAnswerSetCounting(T),
}
impl Mode<Option<usize>> {
    pub fn update(&mut self, with: Option<usize>) {
        match self {
            Self::GoalOriented(_) => *self = Self::GoalOriented(with),
            Self::MaxWeightedFacetCounting(_) => *self = Self::MaxWeightedFacetCounting(with),
            Self::MinWeightedFacetCounting(_) => *self = Self::MinWeightedFacetCounting(with),
            Self::MaxWeightedAnswerSetCounting(_) => {
                *self = Self::MaxWeightedAnswerSetCounting(with)
            }
            Self::MinWeightedAnswerSetCounting(_) => {
                *self = Self::MinWeightedAnswerSetCounting(with)
            }
        }
    }
}
impl fmt::Display for Mode<Option<usize>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GoalOriented(_) => write!(f, "goal oriented (go)"),
            Self::MaxWeightedFacetCounting(_) => {
                write!(f, "strictly goal oriented mode counting facets (max#f)")
            }
            Self::MinWeightedFacetCounting(_) => write!(f, "explore mode counting facets (min#f)"),
            Self::MaxWeightedAnswerSetCounting(_) => {
                write!(
                    f,
                    "strictly goal oriented mode counting answer sets (max#as)"
                )
            }
            Self::MinWeightedAnswerSetCounting(_) => {
                write!(f, "explore mode counting answer sets (min#as)")
            }
        }
    }
}

pub trait Step<T>
where
    T: Clone + Eq + PartialEq,
{
    fn propose_facet(
        &self,
        nav: &mut Navigator,
        active: &mut Vec<String>,
        among: &[String],
    ) -> Option<(String, T)>;
}
impl Step<Option<usize>> for Mode<Option<usize>> {
    fn propose_facet(
        &self,
        nav: &mut Navigator,
        active: &mut Vec<String>,
        among: &[String],
    ) -> Option<(String, Option<usize>)> {
        if among.is_empty() {
            return None;
        }

        match self {
            Self::GoalOriented(_) => among.iter().next().map(|f| (f.to_string(), None)),
            Self::MaxWeightedFacetCounting(prev_count) => {
                let mut counted;
                let bound = Some(0);
                let (mut curr, mut f): (Option<usize>, Option<String>) =
                    (prev_count.map(|c| c - 1).or(Some(usize::MAX)), None);
                for facet in among.iter() {
                    active.push(facet.clone());
                    counted = count(&mut Weight::FacetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x <= y) {
                        curr = counted;
                        f = Some(facet.to_owned());
                    }
                    active.pop();

                    let exc_facet = format!("~{facet}");
                    active.push(exc_facet.clone());
                    counted = count(&mut Weight::FacetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((exc_facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x <= y) {
                        curr = counted;
                        f = Some(exc_facet);
                    }
                    active.pop();
                }

                f.zip(Some(curr))
            }
            Self::MinWeightedFacetCounting(prev_count) => {
                let mut counted;
                let bound = prev_count.map(|c| c - 1).or(Some(usize::MAX));
                let (mut curr, mut f): (Option<usize>, Option<String>) = (Some(0), None);
                for facet in among.iter() {
                    let exc_facet = format!("~{facet}");
                    active.push(exc_facet.clone());
                    counted = count(&mut Weight::FacetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((exc_facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x >= y) {
                        curr = counted;
                        f = Some(exc_facet);
                    }
                    active.pop();

                    active.push(facet.clone());
                    counted = count(&mut Weight::FacetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x >= y) {
                        curr = counted;
                        f = Some(facet.to_owned());
                    }
                    active.pop();
                }

                f.zip(Some(curr))
            }
            Self::MaxWeightedAnswerSetCounting(prev_count) => {
                let mut counted;
                let bound = Some(1);
                let (mut curr, mut f): (Option<usize>, Option<String>) =
                    (prev_count.map(|c| c - 1).or(Some(usize::MAX)), None);
                for facet in among.iter() {
                    active.push(facet.clone());
                    counted = count(&mut Weight::AnswerSetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x <= y) {
                        curr = counted;
                        f = Some(facet.to_owned());
                    }
                    active.pop();

                    counted = prev_count.zip(counted).map(|(x, y)| x - y);
                    if counted == bound {
                        return Some((format!("~{facet}"), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x <= y) {
                        curr = counted;
                        f = Some(format!("~{facet}"));
                    }
                }

                f.zip(Some(curr))
            }
            Self::MinWeightedAnswerSetCounting(prev_count) => {
                let mut counted;
                let bound = prev_count.map(|c| c - 1).or(Some(usize::MAX));
                let (mut curr, mut f): (Option<usize>, Option<String>) = (Some(1), None);
                for facet in among.iter() {
                    let exc_facet = format!("~{facet}");
                    active.push(exc_facet.clone());
                    counted = count(&mut Weight::AnswerSetCounting, nav, active.iter());
                    if counted == bound {
                        return Some((exc_facet.to_string(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x >= y) {
                        curr = counted;
                        f = Some(exc_facet.to_owned());
                    }
                    active.pop();

                    counted = prev_count.zip(counted).map(|(x, y)| x - y);
                    if counted == bound {
                        return Some((facet.to_owned(), bound));
                    }
                    if counted.zip(curr).is_some_and(|(x, y)| x >= y) {
                        curr = counted;
                        f = Some(facet.to_owned());
                    }
                }

                f.zip(Some(curr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use savan::lex::repr;
    use savan::nav::{errors::*, facets::*, Navigator};

    const TINY: &'static str = "a;b. c;d :- b. e.";
    const Q8: &'static str = "
    {q(I ,1..8)} == 1 :- I = 1..8. 
    {q(1..8, J)} == 1 :- J = 1..8. 
    :- {q(D-J, J)} >= 2, D = 2..2*8. 
    :- {q(D+J, J)} >= 2, D = 1-8..8-1.";
    const NONTIGHT: &'static str = "a :- b. b :- a. a :- c. c :- not d. d :- not c.";

    #[test]
    fn tiny_max_fc() -> Result<()> {
        let mut nav = Navigator::new(TINY, vec!["0".to_string()])?;
        for _ in 0..9 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["a", "c", "d", "~b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["b".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["c", "d", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["a".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            assert_eq!(
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among),
                None
            );
        }

        Ok(())
    }

    #[test]
    fn nontight_max_fc() -> Result<()> {
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string(), "--supp-models".to_string()])?;
        for _ in 0..1 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["c", "~d", "~a", "~b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string()])?;
        for _ in 0..4 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["a", "b", "c", "d", "~a", "~b", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn q8_max_fc() -> Result<()> {
        let mut nav = Navigator::new(Q8, vec!["0".to_string()])?;
        for _ in 0..9 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(40));
                assert!(vec!["q(3,3)", "q(6,6)", "q(3,6)", "q(6,3)"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..24 {
            let mut active = vec!["q(6,6)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec![
                    "q(4,1)", "q(3,5)", "q(8,5)", "q(4,7)", "q(5,3)", "q(7,8)", "q(1,3)", "q(1,4)",
                    "q(8,7)", "q(5,8)", "q(3,1)", "q(7,4)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..30 {
            let mut active = vec!["q(2,8)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MaxWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec![
                    "q(4,7)", "q(1,2)", "q(6,5)", "q(8,4)", "q(3,5)", "q(4,5)", "q(8,7)", "q(1,6)",
                    "q(1,3)", "q(5,6)", "q(8,6)", "q(3,6)", "q(3,2)", "q(4,4)", "q(7,5)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn tiny_min_fc() -> Result<()> {
        let mut nav = Navigator::new(TINY, vec!["0".to_string()])?;
        for _ in 0..9 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(6));
                assert!(vec!["~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["b".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["c", "d", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["a".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            assert_eq!(
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among),
                None
            );
        }

        Ok(())
    }

    #[test]
    fn nontight_min_fc() -> Result<()> {
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string(), "--supp-models".to_string()])?;
        for _ in 0..4 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(4));
                assert!(vec!["a", "b", "~c", "d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string()])?;
        for _ in 0..16 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(0));
                assert!(vec!["a", "b", "c", "d", "~a", "~b", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn q8_min_fc() -> Result<()> {
        let mut nav = Navigator::new(Q8, vec!["0".to_string()])?;
        for _ in 0..70 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(126));
                assert!(vec![
                    "~q(3,1)", "~q(8,6)", "~q(4,8)", "~q(5,7)", "~q(3,2)", "~q(5,6)", "~q(2,2)",
                    "~q(2,4)", "~q(3,4)", "~q(4,3)", "~q(1,2)", "~q(7,1)", "~q(5,8)", "~q(7,4)",
                    "~q(4,1)", "~q(2,5)", "~q(4,6)", "~q(2,7)", "~q(7,2)", "~q(6,1)", "~q(5,2)",
                    "~q(6,4)", "~q(7,5)", "~q(3,8)", "~q(1,8)", "~q(4,7)", "~q(6,5)", "~q(7,7)",
                    "~q(3,3)", "~q(6,3)", "~q(3,6)", "~q(1,3)", "~q(1,6)", "~q(7,6)", "~q(2,8)",
                    "~q(1,5)", "~q(8,5)", "~q(1,1)", "~q(8,2)", "~q(4,4)", "~q(6,7)", "~q(7,3)",
                    "~q(2,6)", "~q(4,5)", "~q(6,6)", "~q(5,4)", "~q(2,1)", "~q(8,7)", "~q(2,3)",
                    "~q(3,5)", "~q(1,7)", "~q(8,8)", "~q(8,4)", "~q(5,5)", "~q(6,8)", "~q(8,1)",
                    "~q(8,3)", "~q(4,2)", "~q(1,4)", "~q(5,1)", "~q(3,7)", "~q(5,3)", "~q(6,2)",
                    "~q(7,8)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..22 {
            let mut active = vec!["q(6,6)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(34));
                assert!(vec![
                    "~q(8,7)", "~q(3,5)", "~q(4,1)", "~q(1,4)", "~q(1,3)", "~q(5,8)", "~q(3,1)",
                    "~q(8,5)", "~q(4,7)", "~q(7,4)", "~q(5,3)", "~q(7,8)",
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..6 {
            let mut active = vec!["q(2,8)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = Some(among.len());
            if let Some((f, c)) =
                Mode::MinWeightedFacetCounting(count).propose_facet(&mut nav, &mut active, &among)
            {
                assert_eq!(c, Some(62));
                assert!(vec!["~q(4,5)", "~q(8,7)", "~q(5,6)",].contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn tiny_max_as() -> Result<()> {
        let mut nav = Navigator::new(TINY, vec!["0".to_string()])?;
        for _ in 0..9 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["a", "c", "d", "~b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["b".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["c", "d", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["a".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            assert_eq!(
                Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                    &mut nav,
                    &mut active,
                    &among
                ),
                None
            );
        }

        Ok(())
    }

    #[test]
    fn nontight_max_as() -> Result<()> {
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string(), "--supp-models".to_string()])?;
        for _ in 0..4 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["c", "~d", "~a", "~b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string()])?;
        for _ in 0..16 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["a", "b", "c", "d", "~a", "~b", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn q8_max_as() -> Result<()> {
        let mut nav = Navigator::new(Q8, vec!["0".to_string()])?;
        for _ in 0..16 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(4));
                assert!(vec![
                    "q(3,3)", "q(6,6)", "q(3,6)", "q(6,3)", "q(1,1)", "q(1,8)", "q(8,1)", "q(8,8)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..24 {
            let mut active = vec!["q(6,6)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec![
                    "q(3,1)", "q(1,3)", "q(4,1)", "q(1,4)", "q(4,7)", "q(7,4)", "q(8,5)", "q(5,8)",
                    "q(3,5)", "q(5,3)", "q(7,8)", "q(8,7)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..30 {
            let mut active = vec!["q(2,8)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MaxWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec![
                    "q(4,7)", "q(1,2)", "q(6,5)", "q(8,4)", "q(3,5)", "q(4,5)", "q(8,7)", "q(1,6)",
                    "q(1,3)", "q(5,6)", "q(8,6)", "q(3,6)", "q(3,2)", "q(4,4)", "q(7,5)"
                ]
                .contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn tiny_min_as() -> Result<()> {
        let mut nav = Navigator::new(TINY, vec!["0".to_string()])?;
        for _ in 0..9 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(2));
                assert!(vec!["~a", "~c", "~d", "b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["b".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["c", "d", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        for _ in 0..9 {
            let mut active = vec!["a".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            assert_eq!(
                Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                    &mut nav,
                    &mut active,
                    &among
                ),
                None
            );
        }

        Ok(())
    }

    #[test]
    fn nontight_min_as() -> Result<()> {
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string(), "--supp-models".to_string()])?;
        for _ in 0..4 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(2));
                assert!(vec!["~c", "d", "a", "b"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }
        let mut nav = Navigator::new(NONTIGHT, vec!["0".to_string()])?;
        for _ in 0..16 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(1));
                assert!(vec!["a", "b", "c", "d", "~a", "~b", "~c", "~d"].contains(&f.as_str()));
            } else {
                panic!()
            }
        }

        Ok(())
    }

    #[test]
    fn q8_min_as() -> Result<()> {
        let mut nav = Navigator::new(Q8, vec!["0".to_string()])?;
        for _ in 0..16 {
            let mut active = vec![];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(88));
                assert!(vec![
                    "q(3,3)", "q(6,6)", "q(3,6)", "q(6,3)", "q(1,1)", "q(1,8)", "q(8,1)", "q(8,8)"
                ]
                .iter()
                .map(|f| format!("~{f}"))
                .collect::<Vec<_>>()
                .contains(&f));
            } else {
                panic!()
            }
        }
        for _ in 0..24 {
            let mut active = vec!["q(6,6)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(3));
                assert!(vec![
                    "q(3,1)", "q(1,3)", "q(4,1)", "q(1,4)", "q(4,7)", "q(7,4)", "q(8,5)", "q(5,8)",
                    "q(3,5)", "q(5,3)", "q(7,8)", "q(8,7)"
                ]
                .iter()
                .map(|f| format!("~{f}"))
                .collect::<Vec<_>>()
                .contains(&f));
            } else {
                panic!()
            }
        }
        for _ in 0..30 {
            let mut active = vec!["q(2,8)".to_string()];
            let among = nav
                .facet_inducing_atoms(active.iter())
                .map(|xs| xs.iter().map(|x| repr(*x)).collect::<Vec<_>>())
                .ok_or(NavigatorError::None)?;
            let count = nav.enumerate_solutions_quietly(None, active.iter()).ok();
            if let Some((f, c)) = Mode::MinWeightedAnswerSetCounting(count).propose_facet(
                &mut nav,
                &mut active,
                &among,
            ) {
                assert_eq!(c, Some(7));
                assert!(vec![
                    "q(4,7)", "q(1,2)", "q(6,5)", "q(8,4)", "q(3,5)", "q(4,5)", "q(8,7)", "q(1,6)",
                    "q(1,3)", "q(5,6)", "q(8,6)", "q(3,6)", "q(3,2)", "q(4,4)", "q(7,5)"
                ]
                .iter()
                .map(|f| format!("~{f}"))
                .collect::<Vec<_>>()
                .contains(&f));
            }
        }

        Ok(())
    }
}
