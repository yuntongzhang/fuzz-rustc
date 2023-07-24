#!/bin/bash

## Install dependencies before we can run the actual fuzzer.

# install rust stable
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o install-rustup.sh
chmod +x install-rustup.sh
# use default installation mode
echo "1" | ./install-rustup.sh
rm install-rustup.sh
source "$HOME/.cargo/env"

# install rust nightly
rustup update nightly

