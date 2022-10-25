// Reasoning about the time complexity of the rust compiler
// * Identify patterns known to be slow, in order to notice when something is *unexpectedly* slow
// * Keep the fuzzer from getting bogged down with too many slow testcases

// Small file, but like with the nope module, I want this frequently-changing stuff separate and nonconflicting

use std::cmp::min;
use std::time::Duration;

pub fn expected_dur(input: &str) -> Option<Duration> {
    let size = input.len() as u128;
    let ibytes = input.as_bytes();
    let ct_equals = ibytes.iter().filter(|b| **b == b'=').count() as u128;
    let ct_digits = ibytes.iter().filter(|b| matches!(**b, b'0'..=b'9')).count() as u128;
    let ct_newline = ibytes.iter().filter(|b| **b == b'\n' || **b == b'\r').count() as u128;
    let ct_ampers = ibytes.iter().filter(|b| **b == b'&').count() as u128;

    // let longest_line_len ........... (https://github.com/rust-lang/rust/issues/103429 diagnostic arrow messes) (when not error-format=short)

    if
        // Only fully exclude things that are _frequently_, _annoyingly_ slow *and* we're confident that the
        // we have a way of exclusing (some) hits in a way that doesn't significantly decrease fuzzer coverage
        // See below for issue numbers
        ct_ampers > 300 ||
        //ct_equals > 400 ||   // this is pointless because even 100 hits the thing below
        min(ct_digits, ct_newline) > 300
    {
        eprintln!("timecpx is recommending to skip this input");
        None
    } else {
        let allowance_us = {
            // All programs
            size * size / 4 +  // small allowance for general quadraticness (~4s for a testcase at the soft limit of 4096 bytes)
            5_000 * size + // target 5ms per byte. linear is where we want the compiler to spend time... just not too much
            1_000_000 + // generous allowance for fixed overhead and measurement issues (system load, clock oddities)

            // Programs with specific weirdness. Based on measurements on Jesse's laptop. Times Fudge factor of 5.

            // This is here due to a combination of two bugs:
            // https://github.com/rust-lang/rust/issues/103425 (?) making it really easy to pile up type errors (digits + newlines)
            // https://github.com/rust-lang/rust/issues/103427 making some ident errors linear in the number of type errors (hence size)
            // Measured at 6.6 second for 30 type errors and 4 function names. a function call requires 2 bytes (unclosed parens lol)
            150 * min(ct_digits, ct_newline) * size +

            // Actually worse than quadratic, but we nope'd out early for high ampers counts
            // measured at 1us per ampers squared
            // https://github.com/rust-lang/rust/issues/103421
            5 * ct_ampers * ct_ampers +

            // https://github.com/rust-lang/rust/issues/103411
            // "1=();" meaused at 75ms per repetition
            300_000 * ct_equals
        };

        // Mul some factors like computer speed and whether debug assertions are enabled?
        // Slower getting started? Is it the sancov?

        if allowance_us > 30_000_000 {
            eprintln!("timecpx is concerned this testcase could take {} seconds", (allowance_us as f64) / 1_000_000.0);
            None
        } else {
            Some(Duration::from_micros(allowance_us as u64))
        }
    }
}

pub fn check_dur(elapsed: Duration, allowance: Duration) {
    if elapsed > allowance {
        eprintln!(
            "Compile time of {:?} exceeded the input-dependent allowance of {:?}",
            elapsed, allowance
        );
        // Stash it somewhere? With a sha hash??
    }
}
