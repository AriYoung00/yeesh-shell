# YEESH
## Yeesh is an Extraordinarily Exquisite Shell

Yeesh is an experimental shell written in Rust, with an embedded
python interpreter which is to be used for scripting. 

Arbitrary Python snippets will be runnable at the command line.
Shell variables will be addressable in YEESH script using
the `$` prefix. Additionally, arbitrary shell commands will be
runnable in YEESH script by using the `${<command>}` syntax.

Shell commands and variables are accessed by communicating 

This syntax takes heavy inspiration from 