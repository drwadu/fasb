use regex::Regex;
use savan::nav::{
    weights::{count, Weight},
    Navigator,
};

pub trait Significance {
    fn significance(
        &mut self,
        route: &[String],
        y: String,
        facet_inducing_atoms: &[String],
        re: Regex,
    );
}

impl Significance for Navigator {
    fn significance(
        &mut self,
        route: &[String],
        y: String,
        facet_inducing_atoms: &[String],
        re: Regex,
    ) {
        let mut ctx = route.to_vec();
        ctx.push(y.clone());

        let fc = count(&mut Weight::FacetCounting, self, ctx.iter()).unwrap() as f32;

        if fc == 0.0 {
            if self
                .enumerate_solutions_quietly(Some(1), ctx.iter())
                .is_ok_and(|n| n < 1)
            {
                println!("no facets, no answer set");
            } else {
                println!("no facets, unique answer set");
            }
        } else {
            println!(" inc   exc");
            for a in facet_inducing_atoms.iter().filter(|f| re.is_match(f)) {
                let fc_a = unsafe {
                    count(
                        &mut Weight::FacetCounting,
                        self,
                        ctx.iter().chain([a.clone()].iter()),
                    )
                    .unwrap_unchecked() as f32
                };

                let fc_a_exc = unsafe {
                    count(
                        &mut Weight::FacetCounting,
                        self,
                        ctx.iter().chain([format!("~{a}")].iter()),
                    )
                    .unwrap_unchecked() as f32
                };

                println!("{:.3} {:.3} {a}", 1.0 - (fc_a / fc), 1.0 - (fc_a_exc / fc));
            }
        }
    }
}
