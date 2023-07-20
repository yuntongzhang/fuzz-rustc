#!/bin/bash

## Install dependencies before we can run the actual fuzzer.

# install rust stable
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# install rust nightly
rustup update nightly
