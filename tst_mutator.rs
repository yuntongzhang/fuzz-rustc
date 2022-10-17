// This isn't a test module because:
// * That seems to mess up the compiler caching
// * This is a mix of exploration and regression-testing
// * It's hard to regression-test random coe
// * That seems to run the code in a different context that makes panics behave differently, and
//   a large part of what we're trying to do here is ensure that no panics escape from 
//   deep in the compiler and stop the fuzzer during mutation.

use crate::mutator;
use rand::{rngs::StdRng, SeedableRng};

fn tt(source: &'static str, expected_outputs: &'static [&'static str]) {
    println!("");
    println!("{}", source);
    println!("");
    const SHOW_ITERS: usize = 10;
    const ITERS: usize = 10000;
    let mut r = StdRng::seed_from_u64(0);
    let s = mutator::ProgramMutator::new(source.to_string());

    // Show spans, if we have them
    println!("  {} spans:", s.ts.len());
    for i in &s.ts {
        println!("    {}: {}..{} [{}]", i.tag, i.lo, i.hi, source.get(i.lo .. i.hi).unwrap());
    }

    // Print a few (SHOW_ITERS) results.
    // Compute many (ITER) results and count how many of them match each of the expected_outputs.
    let mut cts = vec![0; expected_outputs.len()];
    let mut error_ct = 0;
    println!("  First {} mutations:", SHOW_ITERS);
    for i in 0..ITERS {
        let m = s.random_mutation(&mut r);
        if let Ok(m) = m {
            if i < SHOW_ITERS {
                println!("    #{}: {}", i, m);
            }
            for i in 0 .. expected_outputs.len() {
                if expected_outputs[i] == m {
                    cts[i] += 1;
                    break;
                }
            }
        } else { 
            error_ct += 1;
        }
    }
    for i in 0 .. expected_outputs.len() {
        println!("Created {}/{} of: {}", cts[i], ITERS, expected_outputs[i]);
    }
    if error_ct > 0 {
        println!("... and {} errors", error_ct);
    }
}


pub fn exercise_mutator() -> ! {
    tt("fn main() { println!(\"{}\", 1); }", &[]);
    tt("fn main() { println!(\"{}\", 2 + 3); }", &[]);
    tt("fn main() { 5; }", &[]);
    tt("fn main() { 8 + + 9; }", &[]);
    tt("fn main() { 7 ", &[]);
    tt("fn f() -> u32 { } ", &[]);
    
    // Test insertion on an empty program
    tt(
        "",
        &[
            "/",
            "X",
            "()",
        ]
    );

    // Test byte-oriented insertion, including delimiter pairs
    tt(
        "ABC",
        &[
            "AXC",
            "AC",
            "AB[]C",
            "A{B}C",
            "{ABC}",
        ]
    );

    // Test insertion. Certain spots should be more likely to be chosen as insertion points.
    tt(
        "fn main() {}",
        &[
            "Xfn main() {}",
            "fXn main() {}",
            "fnX main() {}",
            "fn Xmain() {}",
            "fn main() {X}",
            "fn main() {}X",
        ]
    );

    // Test swapping and stuff
    tt(
        "fn peh(b: bool) -> u32 { match b { false => 0, _ => 1 } }",
        &[
            // Copy the first pattern to the second
            "fn peh(b: bool) -> u32 { match b { false => 0, false => 1 } }",
            // Copy the second pattern to the first
            "fn peh(b: bool) -> u32 { match b { _ => 0, _ => 1 } }",
            // Swap the arms
            "fn peh(b: bool) -> u32 { match b { _ => 1, false => 0 } }",
            // Swap the results
            "fn peh(b: bool) -> u32 { match b { false => 1, _ => 0 } }",
            // Swap the patterns
            "fn peh(b: bool) -> u32 { match b { _ => 0, false => 1 } }",
        ]
    );

    tt(
        "fn main() { std::str::from_utf8(&[65]).unwrap(); }",
        &[]
    );

    // Swap generic params
    tt(
        "fn _k<'a, T>(_t: T) { }",
        &["fn _k<T, 'a>(_t: T) { }"]
    );

    tt(
        "fn main() { let _u = 4; }",
        &[]
    );

    // This one makes the some part of rustc sad, so the parser doesn't give me an AST
    tt("fn main() { 6; } #syntaxerrors#", &[]);

    // This one makes the lexer fail, so we can't even create a parser
    tt("}", &[]);

    // This one is why we need catch_unwind
    tt("use { f() }", &[]);

    // This one is why we need `parse_sess.span_diagnostic.reset_err_count()` after parsing
    // (catch_unwind fails to catch it for some reason!)
    tt("impl<U> Foo<U", &[]);

    println!("Done!");
    std::process::abort();
}
