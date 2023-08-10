#!/usr/bin/python3

import os
import argparse
import subprocess
from os.path import join as pjoin
from tqdm import tqdm
import glob

BUILD_DIR = "/home/yuntong/fuzz-rustc/rust/build"
INSTALL_DIR = "/home/yuntong/fuzz-rustc/rust/install"

RUSTC = pjoin(INSTALL_DIR, "bin", "rustc")
RUSTC_DRIVER = pjoin(INSTALL_DIR, "lib", "librustc_driver-ff276bdba0ed44a5.so")

LLVM_DIR = pjoin(BUILD_DIR, "x86_64-unknown-linux-gnu", "llvm", "bin")
LLVM_COV = pjoin(LLVM_DIR, "llvm-cov")
LLVM_PROFDATA = pjoin(LLVM_DIR, "llvm-profdata")


def remove_raw_files(result_dir: str) -> None:
    for f in glob.glob(result_dir + "/default_*.profraw"):
        os.remove(f)

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compute coverage of a test-suite for rustc"
    )
    parser.add_argument("--corpus", required=True, help="Path to the test corpus")
    parser.add_argument(
        "--result-dir",
        required=True,
        help="Path to the directory where intermediate and final results are stored",
    )

    parsed_args = parser.parse_args()
    corpus_dir = parsed_args.corpus
    result_dir = parsed_args.result_dir

    if not os.path.isabs(corpus_dir):
        corpus_dir = os.path.abspath(corpus_dir)

    if not os.path.isabs(result_dir):
        result_dir = os.path.abspath(result_dir)

    if not os.path.isdir(result_dir):
        os.makedirs(result_dir)
    
    tests = [pjoin(corpus_dir, f) for f in os.listdir(corpus_dir)]

    aggregated_data_file = pjoin(result_dir, "aggregated.profdata")

    os.chdir(result_dir)
    batch_size = 20
    # (1) run compiler against the test suite to produce profraw files
    for index, test in enumerate(tqdm(tests, ncols=100, desc="Running rustc on each test in corpus...", ascii=False)):
        cmd = f"{RUSTC} {test}"
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,  shell=True)
        if index % batch_size == 0:
            # merge files when every batch of raw data files are produced
            if os.path.isfile(aggregated_data_file):
                cmd = f"{LLVM_PROFDATA} merge -sparse {result_dir}/default_*.profraw {aggregated_data_file} -o {aggregated_data_file}"
            else:
                cmd = f"{LLVM_PROFDATA} merge -sparse {result_dir}/default_*.profraw  -o {aggregated_data_file}"
            subprocess.run(cmd, shell=True)
            # remove the raw files, because they take a lot of disk space
            remove_raw_files(result_dir)
             
    # (2) After producing all raw files, merge the leftover one
    cmd = f"{LLVM_PROFDATA} merge -sparse {result_dir}/default_*.profraw {aggregated_data_file}  -o {aggregated_data_file}"
    subprocess.run(cmd, shell=True)
    remove_raw_files(result_dir)

    # (3) produce coverage report
    cmd = f"{LLVM_COV} report "
    cmd += f"--instr-profile={aggregated_data_file} "
    cmd += f"--object {RUSTC} "
    cmd += f"--object {RUSTC_DRIVER} "
    # skip the source files in .cargo/registry directory (which are dependencies)
    cmd += f"--ignore-filename-regex=/.cargo/registry"
    cp = subprocess.run(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, shell=True
    )
    result_content = cp.stdout.strip().split("\n")
    summary_line = result_content[-1].strip().split()

    region_total = int(summary_line[1])
    region_missed = int(summary_line[2])
    region_cov = float(summary_line[3].strip("%"))
    func_total = int(summary_line[4])
    func_missed = int(summary_line[5])
    func_cov = float(summary_line[6].strip("%"))
    line_total = int(summary_line[7])
    line_missed = int(summary_line[8])
    line_cov = float(summary_line[9].strip("%"))

    # print summary results
    print(
        f"Region coverage: {region_cov}% ({region_total - region_missed}/{region_total})"
    )
    print(f"Function coverage: {func_cov}% ({func_total - func_missed}/{func_total})")
    print(f"Line coverage: {line_cov}% ({line_total - line_missed}/{line_total})")


if __name__ == "__main__":
    main()
