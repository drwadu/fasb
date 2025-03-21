use savan::nav::{facets::Facets, Navigator};

#[derive(Debug, Clone)]
pub struct WeightedFacet {
    facet: String,
    inclusive: bool,
    weight: isize,
}

pub fn weighted_facet_count(
    nav: &mut Navigator,
    route: Vec<String>,
    weighted_fs: Vec<WeightedFacet>,
) -> Option<isize> {
    let mut score = 0;

    let bc = nav
        .brave_consequences(route.clone().iter())
        .map(|xs| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;

    if !bc.is_empty() {
        for x in weighted_fs.iter().filter(|w| !w.inclusive) {
            if !bc.contains(&x.facet) {
                score += x.weight
            }
        }
    } else {
        // unsat
        return Some(score);
    }

    let cc = nav
        .cautious_consequences(route.iter())
        .map(|xs| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;

    for x in weighted_fs.iter().filter(|w| w.inclusive) {
        if cc.contains(&x.facet) {
            score += x.weight
        }
    }

    for x in cc {
        if !bc.contains(&x) {
            score += 1
        }
    }

    Some(score)
}

pub fn parse_weighted_facets_from_file(filename: &str) -> Option<Vec<WeightedFacet>> {
    let mut wfcs = vec![];

    for l in std::fs::read_to_string(filename).ok()?.lines() {
        let mut xs = l.split_whitespace();
        let facet = xs.next().map(|s| s.to_string())?;
        let inclusive = xs.next().map(|s| s != "0")?;
        let weight = xs.next().and_then(|s| s.parse::<isize>().ok())?;

        wfcs.push(WeightedFacet {
            facet,
            inclusive,
            weight,
        });
    }

    Some(wfcs)
}
