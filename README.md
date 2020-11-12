Sfota
-----

Six-five-oh-two assembler library.

Very very WIP, will probably never get finished.

## Grammar
```
letter = "A-Za-z"
digit = "0-9"
hex_digit = digit | "A-Fa-f"
undescore = "_"
nonempty_space = " "+
space = " "*
newline = "\n" | "\r\n"

valid_start = letter+

valid_end = (letter|digit|underscore)*

mnemonic = valid_start valid_end

operand = newline | (space "#" "$" digit{4} newline)

instruction = mnemonic operand

line = (instruction | newline)
```

TODO:
```
label = valid_start valid_end ":"

comment = ";" space anything* newline

instruction_part = instruction space (comment|newline)

label_part = (label | nonempty_space) (instruction_part|newline)

interesting_line = (label|nonempty_space) instruction? space? comment?

line = (interesting_line | newline)
```

## TODOs

### How to implement ZP labels?

Easy solution first: only use ZP addressing when it's already known in the parser that it'll fit.
(So effectively only when using numbers as operands, not labels)

But:
Otherwise iterate code gen until fixpoint? Is that guaranteed to exist?
Create syntactic difference? Square brackets maybe?
IDEA:
Use ZP when:
* possible (8bit in ambiguous circumstances)
* required with (zp),Y
* forced using [] (would also force labels). (So \[(zp,X)\] is legal now?
Fixpoint might exist... so then decide ZP or non-ZP at code generation if label fits?

VASM can mark certain symbols as ZP and optimize based on that...
