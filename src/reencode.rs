use savan::nav::errors::Result;
use savan::nav::Navigator;

#[derive(Debug, Clone, Default)]
pub struct Selection {
    cnf: Vec<Vec<String>>,
    dnf: Vec<Vec<String>>,
    initial: String,
}
impl Selection {
    pub fn new() -> Self {
        Selection::default()
    }
    pub fn select(&self, nav: &mut Navigator) -> Result<()> {
        for rule in self.cnf.iter().map(|clause| {
            format!(
                ":- {}.",
                clause
                    .iter()
                    .map(|lit| match lit.starts_with('~') {
                        true => lit[1..].to_owned(),
                        _ => format!("not {lit}"),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }) {
            nav.add_rule(rule)?
        }
        let mut n_ts = 0;
        for rule in self.dnf.iter().enumerate().map(|(i, clause)| {
            format!(
                "t{:?} :- {}.",
                i,
                clause
                    .iter()
                    .map(|lit| lit.replace("~", "not "))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }) {
            nav.add_rule(rule)?;
            n_ts += 1;
        }
        if n_ts > 0 {
            let r = format!(
                ":- {}.",
                (0..n_ts)
                    .map(|i| format!("not t{:?}", i))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            nav.add_rule(r)?;
        }

        Ok(())
    }
    pub fn add_to_cnf(&mut self, clause: Vec<String>) {
        self.cnf.push(clause)
    }
    pub fn add_to_dnf(&mut self, clause: Vec<String>) {
        self.dnf.push(clause)
    }
}
