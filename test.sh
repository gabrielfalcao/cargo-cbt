#!/usr/bin/env bash
##
## #""""""""#                     dP
## #mmm  mmm#                     88
## ####  #### .d8888b. .d8888b. d8888P
## ####  #### 88ooood8 Y8ooooo.   88
## ####  #### 88.  ...       88   88
## ####  #### `88888P' `88888P'   dP
## ##########
##

set -e

cargo run > run.log
cargo install --path .
cargo cbt > cbt.log
