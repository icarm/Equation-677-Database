#!/bin/bash

cargo b --release

function f() {
    cargo r --release &> /dev/null
}

for i in {1..10}
do
    time f
    sleep 3
done
