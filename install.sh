#!/bin/bash

## Install dependencies before we can run the actual fuzzer.

# install rust stable
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o install-rustup.sh
chmod +x install-rustup.sh
# use default installation mode
./install-rustup.sh -y
rm install-rustup.sh
source "$HOME/.cargo/env"

# install rust nightly
rustup install nightly-2023-07-21

