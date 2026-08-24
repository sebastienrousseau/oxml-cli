#!/usr/bin/env bash
#
# XSD validation. The schema comes first because the document is the
# argument that defaults to standard input.
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

expect "a conforming document" 0 'valid' \
  -- validate "$DATA/schema.xsd" "$DATA/catalogue.xml"

finish
