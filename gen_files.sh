#!/bin/sh

rm -Rf test
mkdir test
cd test

xargs touch < ../test_files
