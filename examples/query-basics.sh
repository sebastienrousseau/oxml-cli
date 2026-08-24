#!/usr/bin/env bash
#
# Every output mode of `oxml query`.
#
# The default prints a summary of each matched node; `-t` prints the
# text; `-c` prints only how many matched. Expressions that produce a
# number, a string or a boolean print that value directly.
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

expect "node summaries, one per line" 0 \
'<title>Dune</title>
<title>Germinal</title>' -- query '//title' "$DATA/catalogue.xml"

expect "text only" 0 \
'Dune
Germinal' -- query -t '//title' "$DATA/catalogue.xml"

expect "count only" 0 '2' -- query -c '//book' "$DATA/catalogue.xml"

# A function returning a number prints the number, not a node.
expect "count() as an expression" 0 '2' \
  -- query 'count(//book)' "$DATA/catalogue.xml"

expect "sum() over an attribute-free element" 0 '17.49' \
  -- query 'sum(//price)' "$DATA/catalogue.xml"

# Predicates work as XPath specifies.
expect "a predicate on an attribute" 0 'Dune' \
  -- query -t '//book[@lang="en"]/title' "$DATA/catalogue.xml"

# Attributes are nodes, so the attribute axis has a string-value.
expect "the attribute axis" 0 'en
fr' -- query -t '//book/@lang' "$DATA/catalogue.xml"

# An unprefixed name test matches only nodes in no namespace, which is
# what XPath 1.0 specifies -- so `//@isbn` no longer finds the
# namespaced attribute.
expect "an unprefixed name test skips namespaced nodes" 1 '' \
  -- query -t '//@isbn' "$DATA/catalogue.xml"

# Selecting by namespace. This was skipped until oxml 0.0.4, because
# `namespace-uri()` returned the empty string for every attribute node.
expect "selecting an attribute by namespace URI" 0 '978-0441013593
978-2070413119' -- query -t \
  "//@*[namespace-uri()='urn:example:meta']" "$DATA/catalogue.xml"

# And with a prefix bound on the command line, which is what --ns is
# for: from 0.0.4 an unbound prefix is an error rather than a silent
# match on the local part.
expect "a bound prefix selects only that namespace" 0 '978-0441013593
978-2070413119' -- query -n m=urn:example:meta -t '//@m:isbn' \
  "$DATA/catalogue.xml"

finish
