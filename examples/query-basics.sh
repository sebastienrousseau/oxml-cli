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

# Namespace prefixes are NOT resolved in a name test: `//@isbn` and
# `//@m:isbn` both select the namespaced attribute. This is a known
# defect in the library.
expect "a name test ignores the prefix" 0 '978-0441013593
978-2070413119' -- query -t '//@isbn' "$DATA/catalogue.xml"

# Selecting by namespace instead needs `namespace-uri()` to work on
# attribute nodes, which is fixed in oxml 0.0.4. Until this crate
# depends on it, the assertion is skipped rather than deleted -- a
# commented-out test is one nobody reinstates.
if [[ "${OXML_NAMESPACE_FIX:-0}" == "1" ]]; then
  expect "selecting an attribute by namespace URI" 0 '978-0441013593
978-2070413119' -- query -t \
    "//@*[namespace-uri()='urn:example:meta']" "$DATA/catalogue.xml"
else
  echo "skip: selecting by namespace URI (needs oxml 0.0.4)"
fi

finish
