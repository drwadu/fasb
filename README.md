![build workflow](https://github.com/drwadu/fasb/actions/workflows/build.yml/badge.svg)
![test workflow](https://github.com/drwadu/fasb/actions/workflows/test.yml/badge.svg)
# fasb
Rust implementation of the **f**aceted **a**nswer **s**et **b**rowser from https://doi.org/10.1609/aaai.v36i5.20506.

fasb is a REPL system implemented on top of the [clingo](https://github.com/potassco/clingo) solver. It enables answer set navigation alongside quantitative reasoning.

## quickstart
```bash
$ cargo install fasb
$ fasb 0 program.lp
fasb v0.1.0
42930d520670354cfb84ded47e54142559c70e8cd6b36d6eb2b1a24433adc78f
:: ! 2        -- enumerate at most 2 answer sets under current route 
solution 1:
a e
solution 2:
b d e
found 2
:: ?          -- query facets
b d c a
:: #!!        -- query weights based on answer set counting
0.3333 2 b    -- [reduces # by] [remaining #] [facet]
0.6667 1 d
0.3333 2 ~d
0.6667 1 c
0.3333 2 ~c
0.6667 1 a
0.3333 2 ~a
:: ' max#f    -- use facet-counting strictly goal-oriented mode 
:: $$         -- perform step (causing highest uncertainty reduction)
1.0000 0 d    -- activated facet `d` (reduced facet count by 100%)
:: @          -- query current route
d
:: !          -- enumerate all answer sets under current route
solution 1:
b d e
found 1
```

## install
1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) 
2. `cargo install fasb`
## build
1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) 
2. `cd fasb && cargo build -r`

## commands
* `+ args` ... activate args=[facets a ~a]         
   * e.g.: activate +a and -b: `+ a ~b`         
* `' arg` ... select arg=[navigation mode] among {min,max}#{a,f}                     
* `$$` ... perform next step                          
* quit                                          ->  :q
* next step                                     ->  $
* facet count                                   ->  #?
* facet counts under each facets                ->  #??
* answer set count under route                  ->  #!
* answer set counts under each current facets   ->  #!!
* route                                         ->  @
* facets                                        ->  ?
* deactivate previous facet                     ->  -
* deactivate all facets                         ->  --
