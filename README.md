![build workflow](https://github.com/drwadu/fasb/actions/workflows/build.yml/badge.svg)
![test workflow](https://github.com/drwadu/fasb/actions/workflows/test.yml/badge.svg)
# fasb
Implementation of the **f**aceted **a**nswer **s**et **b**rowser, first metioned in https://doi.org/10.1609/aaai.v36i5.20506.

fasb is a REPL system implemented on top of the [clingo](https://github.com/potassco/clingo) solver. It enables answer set navigation alongside quantitative reasoning.

## quickstart
fasb as a REPL:
```
$ fasb program.lp 0
fasb v0.1.0
42930d520670354cfb84ded47e54142559c70e8cd6b36d6eb2b1a24433adc78f
:: ! 2        -- enumerate up to 2 answer sets
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
fasb as an interpreter:
```
$ cat script.fsb
! 1                  -- output 1 answer set
#?                   -- query facet count        
\ != #f 0 | $$ . ! 2 -- while condition | command . command
@                    -- display route                  
$ fasb program.lp 0 srcipt.fsb
fasb v0.1.0
42930d520670354cfb84ded47e54142559c70e8cd6b36d6eb2b1a24433adc78f
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
  * facet=[p|~p] 
   * e.g.: activate +a and -b: `+ a ~b`         
 * `-` ... deactivate previously activated facet                   
* `--` ... deactivate all facets
* `? regex` ... display current facets matching regex
* `@` ... query current route
* `' arg` ... select arg=[navigation mode] among 
{min,max}#{a,f}                     
  *  by default goal-oriented
  * min* ... explore 
   * max* ... strictly goal-oriented 
   * *#a ... answer set counting 
   * *#f ... facet counting 
   * for supported model counting use *#a and --supp-models flag at start up (fasb 0 program.lp --supp-models)
*  `$ regex` ... query proposed next step in selected mode among facets matching regex                          
* `#?` ... query facet count
* `#!` ... query answer set count 
* `$$ regex` ... perform next step in selected mode among facets matching regex                          
* `#?? regex` ... query facet counts (weights) under each facets matching regex
* `#!! regex` ... query answer set counts (weights) under each facets matching regex
* `man` ... display brief manual
* `:q` ... exit fasb 
