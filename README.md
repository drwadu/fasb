[![Crates.io](https://img.shields.io/crates/v/fasb?label=crates.io%20%28bin%29)](https://crates.io/crates/fasb)
![build workflow](https://github.com/drwadu/fasb/actions/workflows/build.yml/badge.svg)
![test workflow](https://github.com/drwadu/fasb/actions/workflows/test.yml/badge.svg)
# fasb
Implementation of the **f**aceted **a**nswer **s**et **b**rowser, introduced in https://doi.org/10.1609/aaai.v36i5.20506.

fasb is a REPL system implemented on top of the [clingo](https://github.com/potassco/clingo) solver. 
It enables answer set navigation alongside quantitative reasoning.

fasb also implements a basic method for compressing a huge amount of answer sets into representative ones. 
More on representative answer sets can be found in https://ebooks.iospress.nl/doi/10.3233/FAIA230280.

## quickstart
fasb as a REPL:
```
$ fasb program.lp 0
fasb v0.1.2
:: ! 2         -- enumerate up to 2 answer sets
solution 1:
a e
solution 2:
b d e
found 2
:: ?           -- query facets
b d c a
:: #!!         -- query weights based on answer set counting
0.3333 2 b     -- [reduces # by] [remaining #] [facet]
0.6667 1 d
0.3333 2 ~d
0.6667 1 c
0.3333 2 ~c
0.6667 1 a
0.3333 2 ~a
:: ' max#f     -- use facet-counting strictly goal-oriented mode 
:: $$          -- perform step (causing highest uncertainty reduction)
1.0000 0 d     -- activated facet `d` (reduced facet count by 100%)
:: @           -- query current route
d
:: !           -- enumerate all answer sets under current route
solution 1:
b d e
found 1
:: --          -- clear route
:: #!          -- query answer set count
3
:: > a|b&c|d   -- declare cnf query: (a or b) and (c or d)
```
fasb as an interpreter:
```
$ cat script.fsb
! 1                  -- output 1 answer set
#?                   -- query facet count        
\ != #f 0 | $$ . ! 2 -- while condition | command . command
@                    -- display route                  
$ fasb program.lp 0 srcipt.fsb
fasb v0.1.2
:: ! 1
solution 1:
a e
found 1
:: #?
8
:: \ != #f 0 | $$ . ! 2
_ _ b
solution 1:
b d e
solution 2:
b c e
found 2
_ _ c
solution 1:
b c e
found 1
:: @
b c
```

## install
1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) 
2. `cargo install fasb`
## build
1. [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) 
2. `cd fasb && cargo build -r`

## usage
`fasb program [clingo flags] [script]`

Apart from being a REPL system, fasb can also be used as an interpreter of instructions, which will be performed line by line. To use fasb as an interpreter add the feature flag `--feature interpreter` when installing or building. When using the interpreter, provide a script.

The designated syntax for regular expressions (regex) can be found [here](https://docs.rs/regex/latest/regex/).

### commands
* `\ condition | instructions` ... loop '.' seperated instructions while condition={!=,<,<=,>,>=}\s^\d+$\s{#a,#f,#r} where
   * #a ... answer set count
   * #f ... facet count
   * #r ... size of current route 
* `+ args` ... activate args=[whitespace seperated facets]         
  * facet=[a|~a] 
   * e.g.: activate +a and -b: `+ a ~b`         
* `> query` ... declare cnf with `|`-seperated literals and `&`-seperated clauses          
  * literal=[l|~l] 
   * e.g.: `> a|~b&~a|b`         
 * `-` ... deactivate previously activated facet                   
* `--` ... deactivate all facets
* `? regex` ... display current facets matching regex
* `@` ... query current route
* `' arg` ... select navigation mode arg=[{min,max}#{a,f}|go] 
  *  by default goal-oriented (go)
  * min* ... explore 
  * max* ... strictly goal-oriented 
  * *#a ... answer set counting 
  * *#f ... facet counting 
* `! n` ... enumerate n answer sets; if no n is provided, then all answer sets will be printed
* `:! regex` ... print representative answer sets regarding target atoms among facet-inducing atoms that match regex
* `#?` ... query facet count
* `#!` ... query answer set count 
* `#?? regex` ... query facet counts (weights) under each facets matching regex
* `#!! regex` ... query answer set counts (weights) under each facets matching regex
* `:src` ... display underlying program
* `:atoms` ... display atoms (herbrand base)
* `:isatom atom` ... check whether atom belongs to herbrand base
* `man` ... display brief manual
* `:q` ... exit fasb  
