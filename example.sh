#!/bin/bash

set -u

bin="./target/release/parquet2ipc"

ischema="./sample.d/psch.txt"
icsv="./sample.d/input.csv"
iparquet="./sample.d/input.parquet"

export ENV_BATCH_SIZE=8192
export ENV_OFFSET=0

export ENV_PQ_FILENAME="${iparquet}"

geninput(){
  echo generating the input file...

  mkdir -p "./sample.d"
  parquet-fromcsv \
    --has-header \
    --schema "${ischema}" \
    --input-file "${icsv}" \
    --output-file "${iparquet}"
}

run_native(){
    "${bin}"
}

test -f "${iparquet}" || geninput

run_native | xxd | head

run_native | arrow-cat
