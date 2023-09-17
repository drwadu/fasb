pub const PROMPT: &'static str = ":: ";
pub const ACTIVATE_FACETS: &'static str = "+";
pub const SHOW_FACETS: &'static str = "?";
pub const FACET_COUNT: &'static str = "#?";
pub const FACET_COUNTS: &'static str = "#??";
pub const ANSWER_SET_COUNT: &'static str = "#!";
pub const ANSWER_SET_COUNTS: &'static str = "#!!";
pub const ENUMERATE_SOLUTIONS: &'static str = "!";
pub const SHOW_ROUTE: &'static str = "@";
pub const CLEAR_ROUTE: &'static str = "--";
pub const DEL_LAST: &'static str = "-";
pub const CHANGE_MODE: &'static str = "'";
pub const PROPOSE_STEP: &'static str = "$";
pub const TAKE_STEP: &'static str = "$$";
pub const QUIT: &'static str = ":q";

pub(crate) fn manual() {
    println!("activate facets                               ->  {ACTIVATE_FACETS}");
    println!("select navigation mode                        ->  {CHANGE_MODE}");
    println!("perform next step                             ->  {TAKE_STEP}");
    println!("quit                                          ->  {QUIT}");
    println!("next step                                     ->  {PROPOSE_STEP}");
    println!("facet count                                   ->  {FACET_COUNT}");
    println!("facet counts under each facets                ->  {FACET_COUNTS}");
    println!("answer set count under route                  ->  {ANSWER_SET_COUNT}");
    println!("answer set counts under each current facets   ->  {ANSWER_SET_COUNTS}");
    println!("route                                         ->  {SHOW_ROUTE}");
    println!("facets                                        ->  {SHOW_FACETS}");
    println!("deactivate previous facet                     ->  {DEL_LAST}");
    println!("deactivate all facets                         ->  {CLEAR_ROUTE}");
    println!("see documentation for more details");
}
