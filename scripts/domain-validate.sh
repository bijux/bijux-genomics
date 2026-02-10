#!/bin/sh
set -eu

cargo run -p bijux-dna-domain-compiler --bin domain_validate -- --domain-dir domain
