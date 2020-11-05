#Sfota

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
