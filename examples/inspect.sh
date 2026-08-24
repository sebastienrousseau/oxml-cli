#!/usr/bin/env bash
#
# `stats` and `check`: finding out what is in a document, and whether
# it is well-formed at all.
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

expect "stats counts by node kind" 0 \
'nodes       21
elements    7
attributes  4
text        8
comments    1
max depth   4' -- stats "$DATA/catalogue.xml"

expect "check accepts a well-formed document" 0 \
  'well-formed (21 nodes)' -- check "$DATA/catalogue.xml"

# Exit 1 and a line:column position. The position counts characters,
# so it is the column an editor shows.
expect "check rejects a truncated document" 1 \
  'not well-formed at 1:12: at byte 11: input ended unexpectedly' \
  -- check "$DATA/broken.xml"

# Exit 2 is a usage error, distinct from exit 1 for a bad document.
expect "an unknown command is a usage error" 2 \
  'oxml: unknown command `bogus`; try `oxml --help`' -- bogus

finish
