#!/bin/bash

case $1 in
run)
cargo run -- run
  ;;
show)
cargo run -- show
  ;;
restful)
cargo run -- run -c example/restful/config.toml
  ;;
*)
  echo "unknown cmd \"${1}\""
  ;;
esac