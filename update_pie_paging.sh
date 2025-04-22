#!/bin/bash
rm -rf pie-boot/src/paging
cp -rf  page-table-generic/src pie-boot/src/paging
mv pie-boot/src/paging/lib.rs pie-boot/src/paging/mod.rs