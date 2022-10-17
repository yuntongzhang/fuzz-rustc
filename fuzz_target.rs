#![no_main]
#[macro_use] extern crate libfuzzer_sys;
mod mutator;
mod tst_mutator;
use std::sync::Arc;
use std::io::{self, Write};
use std::sync::Mutex;

// A threadsafe buffer. Stolen from rust-lang/rls/blob/78a36bd334b7e51726ea506919e0a4189c416855/rls/src/build/mod.rs
struct TSWriter(Arc<Mutex<Vec<u8>>>);
impl Write for TSWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

/// rustc_driver::Callbacks object that stops before codegen.
pub struct FuzzCallbacks;

impl rustc_driver::Callbacks for FuzzCallbacks {
    fn after_analysis<'tcx>(&mut self,
                            _compiler: &rustc_interface::interface::Compiler,
                            _queries: &'tcx rustc_interface::Queries<'tcx>,) -> rustc_driver::Compilation {
        // Stop before codegen.
        rustc_driver::Compilation::Stop
    }
}

/// FileLoader that holds the contents of a single input file in memory.
/// The idea here is to avoid needing to write to disk.
struct FuzzFileLoader {
    // The contents of the single input file.
    input: String,
}

impl FuzzFileLoader {
    fn new(input: String) -> Self {
        FuzzFileLoader {
            input,
        }
    }
}

// The name of the single input file.
const INPUT_PATH: &str = "fuzz_input.rs";

impl rustc_span::source_map::FileLoader for FuzzFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        std::path::Path::new(INPUT_PATH) == path
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if self.file_exists(path) {
            Ok(self.input.clone())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "tried to open nonexistent file".to_string()))
        }
    }
}

/// CodegenBackend that panics when being called to do any actual codegen.
/// We use this to avoid needing to compile rustc_codegen_llvm.
pub struct NullCodegenBackend;

impl rustc_codegen_ssa::traits::CodegenBackend for NullCodegenBackend {
    fn codegen_crate<'tcx>(&self,
                           _: rustc_middle::ty::TyCtxt<'tcx>,
                           _: rustc_metadata::EncodedMetadata,
                           _: bool) -> std::boxed::Box<(dyn core::any::Any + 'static)> {
        unimplemented!()
    }

    fn join_codegen(
        &self,
        _ongoing_codegen: Box<dyn core::any::Any>,
        _sess: &rustc_session::Session,
        _outputs: &rustc_session::config::OutputFilenames,
    ) -> Result<(rustc_codegen_ssa::CodegenResults,
                 rustc_data_structures::fx::FxHashMap<rustc_middle::dep_graph::WorkProductId,
                                                      rustc_middle::dep_graph::WorkProduct>),
                rustc_errors::ErrorGuaranteed> {
        unimplemented!()
    }

    fn link(
        &self,
        _sess: &rustc_session::Session,
        _codegen_results: rustc_codegen_ssa::CodegenResults,
        _outputs: &rustc_session::config::OutputFilenames,
    ) -> Result<(), rustc_errors::ErrorGuaranteed> {
        unimplemented!()
    }
}


fn rustc_args(input: &str) -> Vec<String> {
    let mut v = vec![
        "rustc".to_string(),
        INPUT_PATH.to_string(),
          "-o".to_string(),
          "dummy_output_file".to_string(),
        ];

    // Pass through certain valid compile flags in the input. (In regression tests, these use e.g. "compile-flags:")

    if input.contains("--edition=2015") || input.contains("edition:2015") {
        v.push("--edition=2015".to_string());
    }
    else if input.contains("--edition=2018") || input.contains("edition:2018") {
        v.push("--edition=2018".to_string());
    }
    else {
        // Always specify some edition
        v.push("--edition=2021".to_string());
    }

    if input.contains("--diagnostic-width=20") {
        v.push("--diagnostic-width=20".to_string());
    }

    if input.contains("--error-format json") || input.contains("--error-format=json") {
        v.push("--error-format=json".to_string());
    }
    else if input.contains("--error-format short") || input.contains("--error-format=short") {
        v.push("--error-format=short".to_string());
    }

    //v.push("-Zverbose".to_string());
    v.push("-L".to_string());
    v.push("FUZZ_RUSTC_LIBRARY_DIR".to_string());
    v
}

pub fn main_fuzz(input: String) {
    //let input_byte_len = input.len();
    let args = rustc_args(&input);
    let file_loader = Box::new(FuzzFileLoader::new(input));
    let mut callbacks = FuzzCallbacks;
    let rse = Arc::default();
    let _result = rustc_driver::catch_fatal_errors(|| {
        let rse_inner = Arc::clone(&rse);
        let mut run_compiler = rustc_driver::RunCompiler::new(&args, &mut callbacks);
        run_compiler.set_file_loader(Some(file_loader));
        run_compiler.set_make_codegen_backend(
            Some(Box::new(|_| {Box::new(NullCodegenBackend)})));
        run_compiler.set_emitter(Some(Box::new(TSWriter(rse_inner))));
        run_compiler.run()
    }).and_then(|result| result);

    let _stderr = Arc::try_unwrap(rse).expect("Compilation is done").into_inner().unwrap();
    //let stderr_byte_len = stderr.len();
    //match String::from_utf8(stderr) {
    //    Ok(s) => {
    //        if stderr_byte_len > 1000 && stderr_byte_len > input_byte_len * 30 && !s.contains("unknown start of token") && !s.contains("character constant must be escaped") && !s.contains("delimiter") {
    //            println!("{}", s);
    //            panic!("Very long output");
    //        }
    //    }
    //    Err(_) => { panic!("Non UTF-8 diagnostics from run_compiler"); }
    //}
}

fuzz_target!(|data: &[u8]| {
    if data.contains(&0x0c) || data.contains(&0x0d) || data.contains(&0x0b) /*|| data.contains (&b'&')*/ {
        return;
    }
    if let Ok(t) = String::from_utf8(data.into()) {
        if t.contains("<") && t.contains("#") && t.contains("[") && t.contains(">>") {
            // Avoid https://github.com/rust-lang/rust/issues/103143
            return;
        }
        main_fuzz(t);
    }
});

fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    if false { tst_mutator::exercise_mutator(); }
    match String::from_utf8(data[0..size].to_vec()) {
        Ok(original_source) => {
            let mut rng = StdRng::seed_from_u64(seed as u64);
            if rng.gen_bool(0.9) && original_source.is_ascii() {
                //println!("Custom mutator");
                let mu = mutator::ProgramMutator::new(original_source);
                let new_source = mu.random_mutation(&mut rng).unwrap();
                let new_data = new_source.as_bytes();
                let new_size = std::cmp::min(max_size, new_data.len());
                data[..new_size].copy_from_slice(&new_data[..new_size]);
                new_data.len()
            } else {
                //println!("Basic mutator");
                //Todo: for non-ascii files, sometimes try the 'reduction' of normalizing the file to ASCII
                libfuzzer_sys::fuzzer_mutate(data, size, max_size)
            }
        }
        _ => {
            // No changes: mutator can't handle it and rustc probably rejects it early anyway
            //eprintln!("How did this non-UTF-8 get in here?");
            //println!("No changes");
            size
        }
    }
});
