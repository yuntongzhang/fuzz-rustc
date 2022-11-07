// References:
// https://github.com/bblfsh/rust-driver/blob/5f33f182d12ec4e39d8ef7406e98d9fefd3ae351/native/src/lib/lib.rs
// https://doc.rust-lang.org/beta/nightly-rustc/rustc_session/parse/struct.ParseSess.html
// https://stackoverflow.com/questions/26575443/how-do-i-use-the-rust-parser-libsyntax-myself
// https://manishearth.github.io/blog/2016/12/02/reflections-on-rusting-trust/
// https://github.com/Medowhill/extract-dependency/blob/a3a6501d0bade5f0015325325d14c91e3baea23f/src/main.rs#L34 (alternative to using default session globals; allows specifying edition)
// Why isn't there a u8 version? Well I guess the file version converts to string so whatever.

use rand::{rngs::StdRng, Rng, prelude::{SliceRandom, IteratorRandom}};


#[derive(Clone, Copy, PartialEq, strum_macros::Display, Debug)]
pub enum SpanTag {
    Ident,
    ForeignItem,
    Item,
    Local,
    Block,
    Stmt,
    Param,
    Arm,
    Pat,
    AnonConst,
    Expr,
    Ty,
    GenericParam,
    Generics,
    WherePredicate,
    AssocItem,
    TraitRef,
    GenericBound,
    PolyTraitRef,
    FieldDef,
    Variant,
    Label,
    Lifetime,
    MacCall,
    Path,
    UseTree,
    PathSegment,
    GenericArgs,
    GenericArg,
    AssocConstraint,
    Attribute,
    Visibility,
    FnRetTy,
    ExprField,
    PatField,
    InlineAsmSym,
}

const MISC_JUNK: [&str; 158] = [
    // Whitespace
    " ",
    "\n",
    "\t",

    // Keywords from rust/compiler/rustc_span/src/symbol.rs
    " {{root}} ",
    " $crate ",
    " _ ",
    " as ",
    " break ",
    " const ",
    " continue ",
    " crate ",
    " else ",
    " enum ",
    " extern ",
    " false ",
    " fn ",
    " for ",
    " if ",
    " impl ",
    " in ",
    " let ",
    " loop ",
    " match ",
    " mod ",
    " move ",
    " mut ",
    " pub ",
    " ref ",
    " return ",
    " self ",
    " Self ",
    " static ",
    " struct ",
    " super ",
    " trait ",
    " true ",
    " type ",
    " unsafe ",
    " use ",
    " where ",
    " while ",
    " abstract ",
    " become ",
    " box ",
    " do ",
    " final ",
    " macro ",
    " override ",
    " priv ",
    " typeof ",
    " unsized ",
    " virtual ",
    " yield ",
    " async ",
    " await ",
    " dyn ",
    " try ",
    " '_ ",
    " 'static ",
    " auto ",
    " catch ",
    " default ",
    " macro_rules! ",
    " raw ",
    " union ",
    " yeet ",

    // Non-keywords that are nevertheleess parsed specially, in some contexts, to recover from certain errors
    " var ",
    " public ",
    " of ",
    " and ",
    " or ",
    " not ",
    " using ",

    // Method names with special diagnostics(?)
    " new ",
    " clone ",
    " map ",
    " and_then ",
    " from ",
    " count ",
    " as_str ",
    " int ",

    // Operators and symbols from https://doc.rust-lang.org/book/appendix-02-operators.html
    "!",
    "!=",
    "%",
    "%=",
    "&",
    "&=",
    "&&",
    "*",
    "*=",
    "+",
    "+=",
    ",",
    "-",
    "-=",
    "->",
    ".",
    "..",
    "..=",
    "...",
    "/",
    "/=",
    ":",
    ";",
    "<<",
    "<<=",
    "<",
    "<=",
    "=",
    "==",
    "=>",
    ">",
    ">=",
    ">>",
    ">>=",
    "@",
    "^",
    "^=",
    "|",
    "|=",
    "||",
    "?",

    // Various forms of literals
    "1_u32",
    "1.0",
    "1f32",
    "r\"...\"",
    "r#\"...\"#",
    "r##\"...\"##",
    "b\"...\"",
    "br\"...\"",
    "br#\"...\"#",
    "br##\"...\"##",
    "'...'",
    "b'...'",

    // Comment out the rest of the line
    "//",
    "//!",
    "///",
    // Complete line comments
    "//\n",
    "//!\n",
    "///\n",
    // Complete block comments
    "/*...*/",
    "/*!...*/",
    "/**...*/",

    // Other stuff
    "#[derive(Copy)]",
    "#[derive(Debug, Copy, Clone)]",
    " ?Sized ",
    " : 'static ",
    " : ?Sized ",
    " &str ",
    " str ",
    " String ",
    " 'a ",

    // Specific compiler flags that the fuzzer notices and passes through
    " /* --edition=2015 */ ",
    " /* --edition=2018 */ ",
    " /* --edition=2021 */ ",
    " /* --diagnostic-width=20 */ ",
    " /* --error-format=json */ ",
    " /* --error-format=short */ ",

    // Consider adding gated features: list in rust/compiler/rustc_feature/src/active.rs 
];

const MISC_CRATE_ATTRIBUTES: [&str; 44] = [
    "#![no_builtins]",  // disables certain optimization patterns
    "#![no_std]",
    "#![no_core]",  // consider disabling no_core, or even adding it to nope.rs, if it causes problems: https://github.com/rust-lang/rust/pull/103003#issuecomment-1277975992
    "#![recursion_limit = \"4\"]",
    "#![type_length_limit = \"4\"]",

    // Let's turn on some allow-by-default lints
    // Seems most sensible to do this at the crate level
    // List from https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    // (These can also be used as crate attributes and perhaps that would be a better use)
    "#![warn(absolute_paths_not_starting_with_crate)]",
    "#![warn(box_pointers)]",
    "#![warn(elided_lifetimes_in_paths)]",
    "#![warn(explicit_outlives_requirements)]",
    "#![warn(ffi_unwind_calls)]",
    "#![warn(fuzzy_provenance_casts)]",
    "#![warn(keyword_idents)]",
    "#![warn(lossy_provenance_casts)]",
    "#![warn(macro_use_extern_crate)]",
    "#![warn(meta_variable_misuse)]",
    "#![warn(missing_abi)]",
    "#![warn(missing_copy_implementations)]",
    "#![warn(missing_debug_implementations)]",
    "#![warn(missing_docs)]",
    "#![warn(must_not_suspend)]",
    "#![warn(non_ascii_idents)]",
    "#![warn(non_exhaustive_omitted_patterns)]",
    "#![warn(noop_method_call)]",
    "#![warn(pointer_structural_match)]",
    "#![warn(rust_2021_incompatible_closure_captures)]",
    "#![warn(rust_2021_incompatible_or_patterns)]",
    "#![warn(rust_2021_prefixes_incompatible_syntax)]",
    "#![warn(rust_2021_prelude_collisions)]",
    "#![warn(single_use_lifetimes)]",
    "#![warn(trivial_casts)]",
    "#![warn(trivial_numeric_casts)]",
    "#![warn(unreachable_pub)]",
    "#![warn(unsafe_code)]",
    "#![warn(unsafe_op_in_unsafe_fn)]",
    "#![warn(unstable_features)]",
    "#![warn(unused_crate_dependencies)]",
    "#![warn(unused_extern_crates)]",
    "#![warn(unused_import_braces)]",
    "#![warn(unused_lifetimes)]",
    "#![warn(unused_macro_rules)]",
    "#![warn(unused_qualifications)]",
    "#![warn(unused_results)]",
    "#![warn(unused_tuple_struct_fields)]",
    "#![warn(variant_size_differences)]",
];

const MISC_ATTRIBUTES: [&str; 16] = [
    // Attributes that go on functions (or closures)
    "#[inline]",
    "#[inline(never)]",
    "#[inline(always)]",
    "#[cold]",
    "#[no_mangle]",
    "#[must_use = \"it's important\"]",

    // Attributes that go on enum & struct definitions
    "#[derive(Copy, Clone)]",
    "#[non_exhaustive]",
    "#[repr(C)]",
    "#[repr(packed(2))]",
    "#[repr(align(8))]",
    "#[repr(C, align(8))]",
    "#[repr(C, u8)]",
    "#[repr(u16)]",
    "#[repr(isize)]",
    "#[repr(transparent)]",  // feature gated
];

fn create_from_thin_air(t: SpanTag, r: &mut StdRng) -> String {
   match t {
        SpanTag::Generics => [
            "", 
            "<T>", 
            "<'a>", 
            "<'a, T>",
        ].choose(r).unwrap().to_string(),
        SpanTag::GenericParam => [
            "", 
            "T", 
            "F", 
            "'a",
        ].choose(r).unwrap().to_string(),
        SpanTag::Ty => [
            "",
            " bool ",
            " u8 ",
            " u16 ",
            " u32 ",
            " u64 ",
            " u128 ",
            " usize ",
            " i8 ",
            " i16 ",
            " i32 ",
            " i64 ",
            " i128 ",
            " isize ",
            "&str",
            "[u32]",
            "[u32; 1]",
            "(u32,)",
            "(u32,u32)",
            "!",  // 'never' type
            "_",  // infer type
            // Traits known to the compiler
            // https://doc.rust-lang.org/reference/special-types-and-traits.html
            // TODO: do these show up in tags other than Ty? Like trait bounds or something
            " Sized ",
            " Send ",
            " Sync ",
            " Copy ",
            " Drop ",
            " Clone ",
            " Debug ",
            " Box ",
            " Box<u32> ",
            " std::rc::Rc<u32> ",
            " std::sync::Arc<u32> ",
            " std::pin::Pin<u32> ",
            " std::cell::UnsafeCell<u32> ",
            " std::marker::PhantomData<u32> ",
            " std::ops::Add ",
            " core::ops::Deref ",
            " core::ops::DerefMut ",
            " std::process::Termination ",
            " std::marker::Unpin ",
            " std::panic::UnwindSafe ",
            " std::panic::RefUnwindSafe ",
        ].choose(r).unwrap().to_string(),
        SpanTag::Expr => [
            "",
            "0",
            "(0)",
            "{0}",
            "{0;0}",
            "todo!()",  // unifies with all types
            "loop{}",   // unifies with all types
            "{let _: u32 = ();}",  // type errors do surprising things to their surrounding contexts (inspired by #103427)
            " ident_error ",  // ident errors too, and in different contexts (inspired by #103181)
        ].choose(r).unwrap().to_string(),
        SpanTag::Pat => [
            "",
            "x",
            "y",
            "1..5",  // exclusive range (requires opt-in to use as a pattern)
            "1..=5",  // inclusive range
            "true",
            "Err(e)",
            "_",  // wildcard pattern
            "..", // "rest" pattern
            "(..)",  // tuple of any size (special pattern)
            "[1, _, _]",
            "Struct{a: 10, b: 'X', c: _}",
        ].choose(r).unwrap().to_string(),
        SpanTag::Path => [
            "",
            "std::str::from_utf8",
            "X",
        ].choose(r).unwrap().to_string(),
        SpanTag::PathSegment => [
            "core",
            "std",
            "str",
            "from_utf8",
            "println",
        ].choose(r).unwrap().to_string(),
        SpanTag::Lifetime => [
            "'static",
            "'a",
            "'_",
        ].choose(r).unwrap().to_string(),
        SpanTag::Attribute => {
            MISC_ATTRIBUTES
        }.choose(r).unwrap().to_string(),
        _ => {
            MISC_JUNK
        }.choose(r).unwrap().to_string(),
   }
}

fn create_from_same_tag(t: SpanTag, old: &str, r: &mut StdRng) -> String {
    match t {
        SpanTag::Ty => {
            match r.gen_range(0..30) {
                0 => "[".to_string() + old + "]",  // slice type
                1 => " Vec<".to_string() + old + "> ",
                2 => " Box<".to_string() + old + "> ",
                3 => " std::rc::Rc<".to_string() + old + "> ",
                4 => " std::sync::Arc<".to_string() + old + "> ",
                5 => " std::pin::Pin<".to_string() + old + "> ",
                6 => " std::cell::UnsafeCell<".to_string() + old + "> ",
                7 => " std::marker::PhantomData<".to_string() + old + "> ",
                8 => old.to_string() + "::" + &create_from_thin_air(SpanTag::Generics, r),
                9 => " dyn ".to_string() + old,
                10 => " & ".to_string() + old,
                11 => " &mut ".to_string() + old,
                12 => " &'a ".to_string() + old,
                13 => " &'static ".to_string() + old,
                14 => " *const ".to_string() + old,
                15 => " *mut ".to_string() + old,
                16 => " impl ".to_string() + old,  // impl (must be followed by a trait?) (only in parameter or return position)
                _ => create_from_thin_air(t, r)
            }
        },
        SpanTag::Expr => {
            match r.gen_range(0..30) {
                0 => "(".to_string() + old + ", " + old + ")",  // tuple, duplicating the expression
                1 => "(".to_string() + old + ",)",  // tuple with one item
                2 => "(".to_string() + old + ")",  // just putting parens around it
                3 => "{".to_string() + old + "}",  // just putting braces around it
                4 => "[".to_string() + old + "]",  // array with one element
                5 => "[".to_string() + old + ", " + old + "]",  // array with two elements, duplicating the expression
                6 => "[".to_string() + old + "; 3]",  // array with three elements, copying the value
                7 => "[0_u32; ".to_string() + old + "]",  // use the expression as an array length (error if not const)
                8 => "(|| ".to_string() + old + ")()",  // closure, executed right away
                9 => "(move || ".to_string() + old + ")()",  // closure which moves its captures, executed right away
                10 => "const {".to_string() + old + "}",  // feature gated btw
                11 => "unsafe {".to_string() + old + "}",
                12 => "async {".to_string() + old + "}",
                13 => "async move {".to_string() + old + "}",
                14 => "loop {".to_string() + old + "}",
                _ => create_from_thin_air(t, r)
            }
        },
        SpanTag::Pat => {
            match r.gen_range(0..32) {
                0 => " ya @ ".to_string() + old,  // bind a new name to the pattern
                1 => old.to_string() + " @ _ ",  // assume we already have a name, and add a pattern to it
                2 => old.to_string() + " | " + old,  // or pattern, but duplicating the exact pattern
                3 => old.to_string() + " | " + &create_from_thin_air(t, r),
                4 => create_from_thin_air(t, r) + " | " + old,
                5 => " ref ".to_string() + old,  // assume it's an identifier ...
                6 => " mut ".to_string() + old,  // assume it's an identifier ...
                7 => " & ".to_string() + old,
                8 => "[".to_string() + old + "]",  // just putting brackets around it, creating a slice pattern
                9 => "[".to_string() + old + ", " + old + "]",  // slice pattern with duplicate pattern
                10..=20 => "(".to_string() + old + ")",  // just putting parens around it
                _ => create_from_thin_air(t, r)
            }
        },
        SpanTag::Item => {
            match r.gen_range(0..12) {
                0 => MISC_ATTRIBUTES.choose(r).unwrap().to_string() + "\n" + old,  // stick an attribute before the item (often a fn)
                // The following is sketchy due to a required order for keywords in function declarations:
                //  `pub`, `default`, `const`, `async`, `unsafe`, `extern`
                1 => "const ".to_string() + old,
                2 => "async ".to_string() + old,
                3 => "unsafe ".to_string() + old,  
                4 => "pub ".to_string() + old, 
                5 => "default ".to_string() + old, 
                6 => "extern ".to_string() + old, 
                _ => create_from_thin_air(t, r)
            }
        }
        _ => create_from_thin_air(t, r)
    }
}

fn ordered_pair<T, F>(mut f: F) -> (T, T) 
where
    F: FnMut() -> T,
    T: PartialOrd
{
    let x = f();
    let y = f();
    if x < y {
        (x, y)
    } else {
        (y, x)
    }
}

#[derive(Debug, PartialEq)]
pub struct SavedSpan {
    pub tag: SpanTag,
    pub lo: usize,
    pub hi: usize
}

impl SavedSpan {
    fn from_rustc_span(s: &rustc_span::Span, t: SpanTag) -> Self {
        SavedSpan { tag: t, lo: s.lo().0 as usize, hi: s.hi().0 as usize }
    }
}

pub struct ProgramMutator {
    pub ts: Vec<SavedSpan>,
    pub src: String,
}

impl ProgramMutator {

    pub fn new(source: String) -> Self {
        // Use Arc/Mutex to smuggle 'ts', our vector of spans, across an unwind boundary.
        // (Is there a better way?)
        use std::sync::{Arc, Mutex};
        let ts_smuggler : Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(vec![]));
        let ts_smuggler_clone = Arc::clone(&ts_smuggler);
        let source_clone = source.clone();
        let panic_res = std::panic::catch_unwind(move || {
            let mut coll = SpanCollector::new();
            coll.collect(source_clone);
            *(ts_smuggler_clone.lock().unwrap()) = coll.ts;
        });
        if panic_res.is_err() {
            //eprintln!("That one panicked!");
        }
        let ts = Arc::try_unwrap(ts_smuggler).unwrap().into_inner().unwrap();
        ProgramMutator { ts: ts, src: source }
    }

    /// Returns a "caret" (position inclusive of 0 and len),
    /// but usually some span boundary.
    fn random_caret(&self, r: &mut StdRng) -> usize {
        if self.ts.is_empty() || r.gen_bool(0.2) {
            r.gen_range(0 ..= self.src.len())
        } else {
            let random_span = self.ts.choose(r).unwrap();
            if r.gen_bool(0.5) {
                random_span.hi
            } else {
                random_span.lo
            }
        }
    }

    /// Returns an "index" (position inclusive of 0 and exclusive of len),
    /// but usually one of the characters on either side of a span boundary.
    fn random_index(&self, r: &mut StdRng) -> usize {
        assert!(!self.src.is_empty());
        let c = self.random_caret(r);
        if c == 0 {
            // We can only go right
            0
        } else if c == self.src.len() {
            // We can only go left
            c - 1
        } else {
            // We can go left or right
            c - r.gen_range(0..=1)
        }
    }

    pub fn random_mutation(&self, r: &mut StdRng) -> Result<String, &'static str> {
        // The categories of mutations available depend on how much information we have about the program
        if !self.ts.is_empty() && r.gen_bool(0.8) {
            self.random_span_aware_mutation(r)
        } else if !self.src.is_empty() && r.gen_bool(0.5) {
            self.random_byte_modification_mutation(r)
        } else {
            self.random_insertion_mutation(r)
        }
    }

    fn random_insertion_mutation(&self, r: &mut StdRng) -> Result<String, &'static str> {
        if r.gen_bool(0.01) {
            // Add a crate attribute at the top
            let out = MISC_CRATE_ATTRIBUTES.choose(r).unwrap().to_string() + &self.src;
            Ok(out)
        } else if r.gen_bool(0.3) {
            // Insert a pair of matched delimiters
            // (They might be unmatched by things between them, or inside strings, but that's fine in a fuzzer)
            // This might make more sense at the token level, esp the heirarachical view, but whatever
            let (first_caret, second_caret) = ordered_pair(|| self.random_caret(r));
            let (first_ins, second_ins) = [
                ("(", ")"),
                ("[", "]"),
                ("{", "}"),
            ].choose(r).unwrap();
            let mut out = "".to_string();
            out += self.src.get(0 .. first_caret).unwrap();
            out += first_ins;
            out += self.src.get(first_caret .. second_caret).unwrap();
            out += second_ins;
            out += self.src.get(second_caret ..).unwrap();
            Ok(out)
        } else {
            // Insert something at a single caret
            let caret = self.random_caret(r);
            let mut out = "".to_string();
            out += self.src.get(0 .. caret).unwrap();
            if r.gen_bool(0.5) {
                // Insert a byte
                out += std::str::from_utf8(&[r.gen_range(32..128)]).unwrap();
            } else {
                // Insert a keyword or something
                out += MISC_JUNK.choose(r).unwrap();
            }
            out += self.src.get(caret ..).unwrap();
            Ok(out)
        }
    }

    fn random_byte_modification_mutation(&self, r: &mut StdRng) -> Result<String, &'static str> {
        let (first_index, second_index) = ordered_pair(|| self.random_index(r));
        let replacement = {
            if r.gen_bool(0.5) {
                // Replace with ANOTHER byte run from the same program (hopefully with different contents)
                let (x, y) = ordered_pair(|| self.random_index(r));
                self.src.get(x .. y + 1).unwrap().to_string()
            } else if r.gen_bool(0.2) {
                // Replace with a single random byte (normal ASCII character)
                std::str::from_utf8(&[r.gen_range(32..128)]).unwrap().to_string()
            } else {
                "".to_string() // Delete
            }
        };

        if first_index < second_index && r.gen_bool(0.1) {
            // Replace several or all occurences of this substring
            Ok(self.replace_random_substring_matches((first_index, second_index), &replacement, r))
        } else {
            // Replace just the one
            Ok(self.src.get(0 .. first_index).unwrap().to_string() + &replacement + self.src.get(second_index + 1 ..).unwrap())
        }
    }

    fn replace_random_substring_matches(&self, (start, end): (usize, usize), replacement: &str, r: &mut StdRng) -> String {
        let orig = self.src.get(start .. end).unwrap();
        let orig_len = end - start;
        let earliest_possible_overlap = if start < orig_len { 0 } else { start - orig_len };
        let pct = *([0.1, 0.5, 0.9, 1.0].choose(r).unwrap());  // not gen_float, in part because testing 1.0 is important
        let victims : Vec<usize> = self.src.match_indices(orig)
            .map(|(s, _)| s)
            .filter(|&s| (earliest_possible_overlap <= s && s <= start) || r.gen_bool(pct))
            .collect();
        assert!(!victims.is_empty());  // we should always have the original match (or one that overlaps it toward the start)
        let mut out = self.src.get(0 .. victims[0]).unwrap().to_string();
        for i in 0 .. victims.len() - 1 {
            out += replacement;
            out += self.src.get(victims[i] + orig_len .. victims[i + 1]).unwrap();
        }
        out += replacement;
        out += self.src.get(victims[victims.len() - 1] + orig_len ..).unwrap();
        out
    }

    fn replace_random_span_matches(&self, sp: &SavedSpan, replacement: &str, r: &mut StdRng) -> String {
        let (require_tag_match, require_substring_match) = match r.gen_range(0..3) {
            0 => (true, true),
            1 => (true, false),
            _ => (false, true)
        };
        let require_nonempty_for_others = r.gen_bool(0.9);
        let pct = *([0.1, 0.5, 0.9, 1.0].choose(r).unwrap());  // not gen_float, in part because testing 1.0 is important
        let orig = self.src.get(sp.lo .. sp.hi).unwrap();
        let possibly_overlapping_potential_victims = self.ts.iter().filter(|&other_sp| (
            (sp == other_sp || r.gen_bool(pct)) &&
            (!require_tag_match || sp.tag == other_sp.tag) &&
            (!require_substring_match || orig == self.src.get(other_sp.lo .. other_sp.hi).unwrap()) &&
            (!require_nonempty_for_others || sp == other_sp || other_sp.lo < other_sp.hi)
        ));
        let mut victims : Vec<&SavedSpan> = Vec::new();
        let mut farth = 0;
        for pv in possibly_overlapping_potential_victims {
            if pv.lo >= farth {
                victims.push(pv);
                farth = pv.hi;
            }
        }
        assert!(!victims.is_empty());
        let mut out = self.src.get(0 .. victims[0].lo).unwrap().to_string();
        for i in 0 .. victims.len() - 1 {
            out += replacement;
            out += self.src.get(victims[i].hi .. victims[i + 1].lo).unwrap();
        }
        out += replacement;
        out += self.src.get(victims[victims.len() - 1].hi ..).unwrap();
        out
    }

    /// This is where the magic happens: copy, swap, or reinvent spans
    fn random_span_aware_mutation(&self, r: &mut StdRng) -> Result<String, &'static str> {
        let span1 = self.ts.choose(r).unwrap();
        let span1_contents = self.src.get(span1.lo .. span1.hi).unwrap();
        
        // Find another span with the same tag but with different text
        let span2 = self.ts.iter()
            .filter(|other| span1.tag == other.tag && span1_contents != self.src.get(other.lo .. other.hi).unwrap())
            .choose(r)
            .unwrap_or_else(|| self.ts.choose(r).unwrap());

        let span2_contents = self.src.get(span2.lo .. span2.hi).unwrap();
        //println!("  {}   {}  -->  {}", span1.tag, span1_contents, span2_contents);

        if span2_contents != span1_contents && span1.hi <= span2.lo && r.gen_bool(0.4) {
            // Swap span1 and span2
            let mut out = "".to_string();
            out += self.src.get(0 .. span1.lo).unwrap();
            out += span2_contents;
            out += self.src.get(span1.hi .. span2.lo).unwrap();
            out += span1_contents;
            out += self.src.get(span2.hi ..).unwrap();
            Ok(out)
        } else {
            // Change span1... or maybe span1 plus several similar spans
            let replacement_contents = {
                if span2_contents != span1_contents && r.gen_bool(0.7) {
                    span2_contents.to_string()
                } else {
                    create_from_same_tag(span1.tag, span1_contents, r)
                }
            };

            if r.gen_bool(0.4) {
                // Replace several spans that are "similar to" span1
                // (Often this ends up only replacing one span anyway, so it doesn't hurt to have the probability high)
                Ok(self.replace_random_span_matches(span1, &replacement_contents, r))
            } else {
                // Replace just span1
                let mut out = "".to_string();
                out += self.src.get(0 .. span1.lo).unwrap();
                out += &replacement_contents;
                out += self.src.get(span1.hi ..).unwrap();
                //println!("    {}", out);
                Ok(out)
            }
        }
    }
}


pub struct SpanCollector {
    pub ts: Vec<SavedSpan>,
}

impl SpanCollector {
    pub fn new() -> Self {
        SpanCollector { ts: vec![] }
    }

    pub fn collect(&mut self, source: String) {
        rustc_span::create_default_session_globals_then(|| {
            let parse_sess = rustc_session::parse::ParseSess::with_silent_emitter(None);
            let fname = rustc_span::FileName::Custom("parsing-for-mutator".to_string());
            // Scope for parser_res so that it can be finished borrowing parse_sess
            {
                let parser_res = rustc_parse::maybe_new_parser_from_source_str(&parse_sess, fname, source.clone());  // can panic
                match parser_res {
                    Ok(mut parser) => {
                        match parser.parse_crate_mod() {
                            Ok(krate) => {
                                //eprintln!("Goooood"); 
                                rustc_ast::visit::walk_crate(self, &krate); 
                            }
                            Err(parser_err) => {
                                //eprintln!("We got a parser, but it returned an error instead of giving us an AST"); 
                                parser_err.cancel();
                            }
                        }
                    }
                    Err(_lexer_diags_i_guess) => { 
                        //eprintln!("We didn't even get a parser"); 
                    }
                }
            }
            // We don't want "flush_delayed" to panic-on-drop
            parse_sess.span_diagnostic.reset_err_count();  
        });
    }
}

// Walk the AST normally, and record spans into self.ts (for AST nodes that have spans)
// This must be kept up to date with rust/compiler/rustc_ast/src/visit.rs, including new methods to override
impl<'ast> rustc_ast::visit::Visitor<'ast> for SpanCollector {
    fn visit_ident(&mut self, ident: rustc_span::symbol::Ident) {
        self.ts.push(SavedSpan::from_rustc_span(&ident.span, SpanTag::Ident));
    }
    fn visit_foreign_item(&mut self, i: &'ast rustc_ast::ForeignItem) {
        self.ts.push(SavedSpan::from_rustc_span(&i.span, SpanTag::ForeignItem));
        rustc_ast::visit::walk_foreign_item(self, i)
    }
    fn visit_item(&mut self, i: &'ast rustc_ast::Item) {
        self.ts.push(SavedSpan::from_rustc_span(&i.span, SpanTag::Item));
        rustc_ast::visit::walk_item(self, i)
    }
    fn visit_local(&mut self, l: &'ast rustc_ast::Local) {
        self.ts.push(SavedSpan::from_rustc_span(&l.span, SpanTag::Local));
        rustc_ast::visit::walk_local(self, l)
    }
    fn visit_block(&mut self, b: &'ast rustc_ast::Block) {
        self.ts.push(SavedSpan::from_rustc_span(&b.span, SpanTag::Block));
        rustc_ast::visit::walk_block(self, b)
    }
    fn visit_stmt(&mut self, s: &'ast rustc_ast::Stmt) {
        self.ts.push(SavedSpan::from_rustc_span(&s.span, SpanTag::Stmt));
        rustc_ast::visit::walk_stmt(self, s)
    }
    fn visit_param(&mut self, param: &'ast rustc_ast::Param) {
        self.ts.push(SavedSpan::from_rustc_span(&param.span, SpanTag::Param));
        rustc_ast::visit::walk_param(self, param)
    }
    fn visit_arm(&mut self, a: &'ast rustc_ast::Arm) {
        self.ts.push(SavedSpan::from_rustc_span(&a.span, SpanTag::Arm));
        rustc_ast::visit::walk_arm(self, a)
    }
    fn visit_pat(&mut self, p: &'ast rustc_ast::Pat) {
        self.ts.push(SavedSpan::from_rustc_span(&p.span, SpanTag::Pat));
        rustc_ast::visit::walk_pat(self, p)
    }
    fn visit_anon_const(&mut self, c: &'ast rustc_ast::AnonConst) {
        self.ts.push(SavedSpan::from_rustc_span(&c.value.span, SpanTag::AnonConst));
        rustc_ast::visit::walk_anon_const(self, c)
    }
    fn visit_expr(&mut self, ex: &'ast rustc_ast::Expr) {
        self.ts.push(SavedSpan::from_rustc_span(&ex.span, SpanTag::Expr));
        rustc_ast::visit::walk_expr(self, ex)
    }
//    fn visit_expr_post(&mut self, _ex: &'ast rustc_ast::Expr) {
//
//    }
    fn visit_ty(&mut self, t: &'ast rustc_ast::Ty) {
        self.ts.push(SavedSpan::from_rustc_span(&t.span, SpanTag::Ty));
        rustc_ast::visit::walk_ty(self, t)
    }
    fn visit_generic_param(&mut self, param: &'ast rustc_ast::GenericParam) {
        self.ts.push(SavedSpan::from_rustc_span(&param.span(), SpanTag::GenericParam));
        rustc_ast::visit::walk_generic_param(self, param)
    }
    fn visit_generics(&mut self, g: &'ast rustc_ast::Generics) {
        self.ts.push(SavedSpan::from_rustc_span(&g.span, SpanTag::Generics));
        rustc_ast::visit::walk_generics(self, g)
    }
    // No span found (ClosureBinder is an enum)
    //fn visit_closure_binder(&mut self, b: &'ast rustc_ast::ClosureBinder) {
    //    rustc_ast::visit::walk_closure_binder(self, b)
    //}
    fn visit_where_predicate(&mut self, p: &'ast rustc_ast::WherePredicate) {
        self.ts.push(SavedSpan::from_rustc_span(&p.span(), SpanTag::WherePredicate));
        rustc_ast::visit::walk_where_predicate(self, p)
    }
    // No span found
    //fn visit_fn(&mut self, fk: rustc_ast::visit::FnKind<'ast>, _: rustc_span::Span, _: rustc_ast::NodeId) {
    //    rustc_ast::visit::walk_fn(self, fk)
    //}
    fn visit_assoc_item(&mut self, i: &'ast rustc_ast::AssocItem, ctxt: rustc_ast::visit::AssocCtxt) {
        self.ts.push(SavedSpan::from_rustc_span(&i.span, SpanTag::AssocItem));
        rustc_ast::visit::walk_assoc_item(self, i, ctxt)
    }
    fn visit_trait_ref(&mut self, t: &'ast rustc_ast::TraitRef) {
        self.ts.push(SavedSpan::from_rustc_span(&t.path.span, SpanTag::TraitRef));
        rustc_ast::visit::walk_trait_ref(self, t)
    }
    fn visit_param_bound(&mut self, bounds: &'ast rustc_ast::GenericBound, _ctxt: rustc_ast::visit::BoundKind) {
        self.ts.push(SavedSpan::from_rustc_span(&bounds.span(), SpanTag::GenericBound));
        rustc_ast::visit::walk_param_bound(self, bounds)
    }
    fn visit_poly_trait_ref(&mut self, t: &'ast rustc_ast::PolyTraitRef) {
        self.ts.push(SavedSpan::from_rustc_span(&t.span, SpanTag::PolyTraitRef));
        rustc_ast::visit::walk_poly_trait_ref(self, t)
    }
    //fn visit_variant_data(&mut self, s: &'ast rustc_ast::VariantData) {
    //    rustc_ast::visit::walk_struct_def(self, s)
    //}
    fn visit_field_def(&mut self, s: &'ast rustc_ast::FieldDef) {
        self.ts.push(SavedSpan::from_rustc_span(&s.span, SpanTag::FieldDef));
        rustc_ast::visit::walk_field_def(self, s)
    }
    // No path found (EnumDef is just a vec of variants. Each one has its own span but we visit it elsewhere, I think.
    //fn visit_enum_def(&mut self, enum_definition: &'ast rustc_ast::EnumDef) {
    //    rustc_ast::visit::walk_enum_def(self, enum_definition)
    //}
    fn visit_variant(&mut self, v: &'ast rustc_ast::Variant) {
        self.ts.push(SavedSpan::from_rustc_span(&v.span, SpanTag::Variant));
        rustc_ast::visit::walk_variant(self, v)
    }
    fn visit_label(&mut self, label: &'ast rustc_ast::Label) {
        self.ts.push(SavedSpan::from_rustc_span(&label.ident.span, SpanTag::Label));
        rustc_ast::visit::walk_label(self, label)
    }
    fn visit_lifetime(&mut self, lifetime: &'ast rustc_ast::Lifetime, _: rustc_ast::visit::LifetimeCtxt) {
        self.ts.push(SavedSpan::from_rustc_span(&lifetime.ident.span, SpanTag::Lifetime));
        rustc_ast::visit::walk_lifetime(self, lifetime)
    }
    fn visit_mac_call(&mut self, mac: &'ast rustc_ast::MacCall) {
        self.ts.push(SavedSpan::from_rustc_span(&mac.span(), SpanTag::MacCall));
        rustc_ast::visit::walk_mac(self, mac)
    }
    //fn visit_mac_def(&mut self, _mac: &'ast rustc_ast::MacroDef, _id: rustc_ast::NodeId) {
    //    // Nothing to do
    //}
    fn visit_path(&mut self, path: &'ast rustc_ast::Path, _id: rustc_ast::NodeId) {
        self.ts.push(SavedSpan::from_rustc_span(&path.span, SpanTag::Path));
        rustc_ast::visit::walk_path(self, path)
    }
    fn visit_use_tree(&mut self, use_tree: &'ast rustc_ast::UseTree, id: rustc_ast::NodeId, _nested: bool) {
        self.ts.push(SavedSpan::from_rustc_span(&use_tree.span, SpanTag::UseTree));
        rustc_ast::visit::walk_use_tree(self, use_tree, id)
    }
    fn visit_path_segment(&mut self, path_segment: &'ast rustc_ast::PathSegment) {
        self.ts.push(SavedSpan::from_rustc_span(&path_segment.span(), SpanTag::PathSegment));
        rustc_ast::visit::walk_path_segment(self, path_segment)
    }
    fn visit_generic_args(&mut self, generic_args: &'ast rustc_ast::GenericArgs) {
        self.ts.push(SavedSpan::from_rustc_span(&generic_args.span(), SpanTag::GenericArgs));
        rustc_ast::visit::walk_generic_args(self, generic_args)
    }
    fn visit_generic_arg(&mut self, generic_arg: &'ast rustc_ast::GenericArg) {
        self.ts.push(SavedSpan::from_rustc_span(&generic_arg.span(), SpanTag::GenericArg));
        rustc_ast::visit::walk_generic_arg(self, generic_arg)
    }
    fn visit_assoc_constraint(&mut self, constraint: &'ast rustc_ast::AssocConstraint) {
        self.ts.push(SavedSpan::from_rustc_span(&constraint.span, SpanTag::AssocConstraint));
        rustc_ast::visit::walk_assoc_constraint(self, constraint)
    }
    fn visit_attribute(&mut self, attr: &'ast rustc_ast::Attribute) {
        self.ts.push(SavedSpan::from_rustc_span(&attr.span, SpanTag::Attribute));
        rustc_ast::visit::walk_attribute(self, attr)
    }
    fn visit_vis(&mut self, vis: &'ast rustc_ast::Visibility) {
        self.ts.push(SavedSpan::from_rustc_span(&vis.span, SpanTag::Visibility));
        rustc_ast::visit::walk_vis(self, vis)
    }
    fn visit_fn_ret_ty(&mut self, ret_ty: &'ast rustc_ast::FnRetTy) {
        self.ts.push(SavedSpan::from_rustc_span(&ret_ty.span(), SpanTag::FnRetTy));
        rustc_ast::visit::walk_fn_ret_ty(self, ret_ty)
    }
    //fn visit_fn_header(&mut self, _header: &'ast rustc_ast::FnHeader) {
    //    // Nothing to do
    //}
    fn visit_expr_field(&mut self, f: &'ast rustc_ast::ExprField) {
        self.ts.push(SavedSpan::from_rustc_span(&f.span, SpanTag::ExprField));
        rustc_ast::visit::walk_expr_field(self, f)
    }
    fn visit_pat_field(&mut self, fp: &'ast rustc_ast::PatField) {
        self.ts.push(SavedSpan::from_rustc_span(&fp.span, SpanTag::PatField));
        rustc_ast::visit::walk_pat_field(self, fp)
    }
    //fn visit_crate(&mut self, krate: &'ast rustc_ast::Crate) {
    //    rustc_ast::visit::walk_crate(self, krate)
    //}
    //fn visit_inline_asm(&mut self, asm: &'ast rustc_ast::InlineAsm) {
    //    rustc_ast::visit::walk_inline_asm(self, asm)
    //}
    fn visit_inline_asm_sym(&mut self, sym: &'ast rustc_ast::InlineAsmSym) {
        self.ts.push(SavedSpan::from_rustc_span(&sym.path.span, SpanTag::InlineAsmSym));
        rustc_ast::visit::walk_inline_asm_sym(self, sym)
    }
}
