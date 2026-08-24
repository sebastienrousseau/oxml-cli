<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Exit codes

| Code | Meaning |
|---|---|
| 0 | Success; for `validate`, the document conforms |
| 1 | The document is invalid, or no nodes matched |
| 2 | A usage or I/O error |

## Why 1 and 2 are different

A shell script asking "does this document contain any errors" wants a
boolean:

```bash
if oxml query -c '//error' build.xml > /dev/null; then
  echo "the build reported errors"
fi
```

That reads correctly only if "no match" is a non-zero exit. So `query`
returns 1 when nothing matched.

But the same script must not treat "you passed a path that does not
exist" as "no errors found". Under `set -e`, a tool that returns 1 for
both turns a typo into a silent pass — the script sees a clean result
and continues.

So a *usage* problem is 2: an unknown command, a missing argument, a
file that cannot be read. Two situations, two codes.

`xmllint --xpath` exits 0 when its expression matches nothing and
writes a message to stderr. That is defensible and it is the opposite
choice; scripts that rely on it need rewriting when they move here.

## Checking the distinction

```bash
$ oxml query '//nothing' catalogue.xml; echo $?
1
$ oxml bogus catalogue.xml; echo $?
2
$ oxml check broken.xml; echo $?
1
$ oxml check catalogue.xml; echo $?
0
```

All four are asserted in [`examples/inspect.sh`](../examples/inspect.sh)
and [`examples/pipeline.sh`](../examples/pipeline.sh), which run in CI.

## Diagnostics go to stderr

Nothing but results appears on stdout, so a pipe stays clean even when
the command fails. That is asserted too.
