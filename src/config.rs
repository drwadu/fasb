pub const PROMPT: &'static str = ":: ";
pub const ACTIVATE_FACETS: &'static str = "+";
pub const SHOW_FACETS: &'static str = "?";
pub const FACET_COUNT: &'static str = "#?";
pub const FACET_COUNTS: &'static str = "#??";
pub const ANSWER_SET_COUNT: &'static str = "#!";
pub const ANSWER_SET_COUNTS: &'static str = "#!!";
pub const ENUMERATE_SOLUTIONS: &'static str = "!";
pub const ENUMERATE_PROJECTED_SOLUTIONS: &'static str = "!*";
pub const SHOW_ROUTE: &'static str = "@";
pub const CLEAR_ROUTE: &'static str = "--";
pub const DEL_LAST: &'static str = "-";
pub const CHANGE_MODE: &'static str = "'";
pub const DISPLAY_MODE: &'static str = ":mode";
pub const PROPOSE_STEP: &'static str = "$";
pub const TAKE_STEP: &'static str = "$$";
pub const SHOW_PROGRAM: &'static str = ":src";
pub const SHOW_ATOMS: &'static str = ":atoms";
pub const IS_ATOM: &'static str = ":isatom";
pub const SOE: &'static str = ":soe";
pub const CONTEXT: &'static str = ">";
pub const SIGNIFICANCE: &'static str = "%";
pub const QUIT: &'static str = ":q";

pub(crate) fn manual() {
    println!("display facet-inducing atoms                                    ->  {SHOW_FACETS}");
    println!("display route                                                   ->  {SHOW_ROUTE}");
    println!("enumerate n=[int] answer sets                                   ->  {ENUMERATE_SOLUTIONS} n");
    println!("activate facets=[whitespace seperated literals, e.g., a ~b]     ->  {ACTIVATE_FACETS} facets ");
    println!("deactivate previous facet                                       ->  {DEL_LAST}");
    println!("deactivate all facets                                           ->  {CLEAR_ROUTE}");
    println!("declare cnf=[e.g., a|~b&c|d] context/query                      ->  {CONTEXT}");
    println!(
        "select navigation mode=[{{{{min,max}}#{{f,a,s}}, go}}]                ->  {CHANGE_MODE}"
    );
    println!("next step in mode                                               ->  {PROPOSE_STEP}");
    println!("perform next step in mode                                       ->  {TAKE_STEP}");
    println!("facet count                                                     ->  {FACET_COUNT}");
    println!("facet counts under each facet                                   ->  {FACET_COUNTS}");
    println!("significance of facets=[regex] for some facet=[a or ~a]         ->  {SIGNIFICANCE}");
    println!(
        "answer set count                                                ->  {ANSWER_SET_COUNT}"
    );
    println!(
        "answer set counts under each facet                              ->  {ANSWER_SET_COUNTS}"
    );
    println!("display program                                                 ->  {SHOW_PROGRAM}");
    println!("display atoms                                                   ->  {SHOW_ATOMS}");
    println!("atom check                                                      ->  {IS_ATOM}");
    println!("display navigation mode                                         ->  {DISPLAY_MODE}");
    println!("quit                                                            ->  {QUIT}");
    println!("see documentation for more details");
}
