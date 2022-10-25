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
    const ITERS: usize = 100000;
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

/// Let's see what kinds of spans we can create, and which inputs create ASTs despite having syntax errors
pub fn test_exploring_the_spans() {
    tt("fn main() { println!(\"{}\", 2 + 3); }", &[]);
    tt("fn main() { 8 + + 9; }", &[]);
    tt("fn main() { 7", &[]);
    tt("fn f() -> u32 { }", &[]);
    tt("fn main() { std::str::from_utf8(&[65]).unwrap(); }", &[]);
    tt("fn main() { let _u = 4; }", &[]);

    tt(
        "const fn call_twice<F>(mut f: F)
         where
             F: ~const FnMut() -> () + ~const std::marker::Destruct,
         {
             f();
             f();
         }",
    &[]);

    tt(
        "async fn service<H, F>(r_handler: Arc<H>) -> Result<http::Response<hyper::Body>, Infallible>
         where
             H: Fn(&str) -> F + Send,
             F: Future<Output = http::Response<hyper::Body>> + Send,
         {
             r_handler(s).await;
             todo!()
         }",
    &[]);
}

pub fn test_do_not_crash() {
    // This one makes the some part of rustc sad, so the parser doesn't give me an AST
    tt("fn main() { 6; } #syntaxerrors#", &[]);

    // This one makes the lexer fail, so we can't even create a parser
    tt("}", &[]);

    // This one is why we need catch_unwind
    tt("use { f() }", &[]);

    // This one is why we need `parse_sess.span_diagnostic.reset_err_count()` after parsing
    // (catch_unwind fails to catch it for some reason!)
    tt("impl<U> Foo<U", &[]);

    // An unclosed literal.... well it doesn't crash, but it puts the error on stderr instead of letting us capture it
    tt("'", &[]);
}

pub fn test_expected_generation() {
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

    // Test copying and swapping spans
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

    // Test swapping generic params
    tt(
        "fn _k<'a, T>(_t: T) { }",
        &["fn _k<T, 'a>(_t: T) { }"]
    );

    // Test byte-level replace multiple
    tt(
        "AAAAA.p",
        &[
            "ppppp.p",  // replace all "A" with "p"
            "ApApp.p",  // replace about half "A" with "p"
            "ppA.p",    // replace all (non-overlapping) "AA" with "p"
        ]
    );

    // Test span-level replace multiple (by tag + string)
    tt(
        "struct Color(u8, u8, u8); struct Spam(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);",
        &[
            // replace half "u8" with "i32" (Ty -> Ty or several other options)
            "struct Color(i32, u8, i32); struct Spam(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);",
            // replace all "u8" with "Spam" (Ident -> Ident)
            "struct Color(Spam, Spam, Spam); struct Spam(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);",
        ]
    );

    // Test span-level replace multiple (by tag)
    tt(
        "struct Smorgasbord(String, u32, i8, (), &'static str, Option<isize>);",
        &[
            // replace all ty with u32 (resolving overlap as outer)
            "struct Smorgasbord(u32, u32, u32, u32, u32, u32);",
            // replace all ty with u32 (resolving overlap as inner - not implemented)
            "struct Smorgasbord(u32, u32, u32, u32, &'static u32, Option<u32>);",
            // replace some with u32
            "struct Smorgasbord(u32, u32, u32, (), &'static str, Option<isize>);",
        ]
    );

    // Test span-initiated but byte-level replace multiple
    tt(
        "fn lftim<'a>(_x: &'a str) { }",
        &[
            // Replace the 'a that is a Lifetime with 'static
            "fn lftim<'a>(_x: &'static str) { }",
            // Replace both instances of 'a with 'static, even though the first is a GenericParam and not a lifetime
            // (replace-all at the byte level based on information from spans)
            "fn lftim<'static>(_x: &'static str) { }",
        ]
    );



}

pub fn exercise_mutator() {
    test_exploring_the_spans();
    test_expected_generation();
    test_do_not_crash();
}
