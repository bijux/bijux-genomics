#!/bin/sh
set -eu

cargo run --bin bijux-dna -- domain validate --domain-dir domain
