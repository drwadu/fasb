pub const PROMPT: &'static str = ":: ";
pub const ACTIVATE_FACETS: &'static str = "+";
pub const SHOW_FACETS: &'static str = "?";
pub const FACET_COUNT: &'static str = "#?";
pub const FACET_COUNTS: &'static str = "#??";
pub const ANSWER_SET_COUNT: &'static str = "#!";
pub const ANSWER_SET_COUNTS: &'static str = "#!!";
pub const ENUMERATE_SOLUTIONS: &'static str = "!";
pub const ENUMERATE_SOLUTIONS_VIZ: &'static str = "V!";
pub const SHOW_ROUTE: &'static str = "@";
pub const CLEAR_ROUTE: &'static str = "--";
pub const DEL_LAST: &'static str = "-";
pub const CHANGE_MODE: &'static str = "'";
pub const PROPOSE_STEP: &'static str = "$";
pub const TAKE_STEP: &'static str = "$$";
pub const ADD_RULE: &'static str = "L+";
pub const ADD_C: &'static str = "C+";
pub const ADD_T: &'static str = "T+";
pub const DELETE_RULE: &'static str = "L-";
pub const SHOW_PROGRAM: &'static str = "L";
pub const SHOW_ATOMS: &'static str = "A";
pub const IS_ATOM: &'static str = "AA";
pub const SOE: &'static str = ":!";
pub const SOE_VERBOSE: &'static str = ":!v";
pub const SOE_VIZ: &'static str = "V:!";
pub const QUIT: &'static str = ":q";

pub const WATERMARK: &'static str = "e4062e46d304db94e142767b7939f39dc193ab182edac6e377e25993d984b402";

pub(crate) fn manual() {
    println!("activate facets                               ->  {ACTIVATE_FACETS}");
    println!("enumerate answer sets                         ->  {ENUMERATE_SOLUTIONS}");
    println!("select navigation mode                        ->  {CHANGE_MODE}");
    println!("perform next step                             ->  {TAKE_STEP}");
    println!("quit                                          ->  {QUIT}");
    println!("next step                                     ->  {PROPOSE_STEP}");
    println!("facet count                                   ->  {FACET_COUNT}");
    println!("facet counts under each facets                ->  {FACET_COUNTS}");
    println!("answer set count under route                  ->  {ANSWER_SET_COUNT}");
    println!("answer set counts under each current facets   ->  {ANSWER_SET_COUNTS}");
    println!("display route                                 ->  {SHOW_ROUTE}");
    println!("display facets                                ->  {SHOW_FACETS}");
    println!("deactivate previous facet                     ->  {DEL_LAST}");
    println!("deactivate all facets                         ->  {CLEAR_ROUTE}");
    println!("deactivate all facets                         ->  {CLEAR_ROUTE}");
    println!("display program                               ->  {SHOW_PROGRAM}");
    println!("display atoms                                 ->  {SHOW_ATOMS}");
    println!("atom check                                    ->  {IS_ATOM}");
    println!("add rule                                      ->  {ADD_RULE}");
    println!("delete rule                                   ->  {DELETE_RULE}");
    println!("see documentation for more details");
}
