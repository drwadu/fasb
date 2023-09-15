#!/bin/bash

for instance in ./$1/*.lp; do
	s=$instance
	a=(${s//// })
	file=${a[3]}
	echo $instance
	perf stat --timeout 600000 bins/fasb $instance 0 $2 &> $3/$file.out
    done
