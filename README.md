# writ

`writ` is a command line slideshow utility. 

It is inspired by [`sent`](https://tools.suckless.org/sent/).

## Who's it for?
Me. However, if you want to use it and slightly modify the formatting,
format variables are stored in the `config.rs` file - modify those and recompile.

## Usage
`writ --input [INPUT FILE] --output [OUTPUT PATH]`

where [INPUT FILE] is a plain text file which is parsed as:
- blank line -> new slide (so, one slide per paragraph)
- line of text -> text
- line starting with `@` -> path to an image
- line starting with `$` -> a LaTeX equation
- line starting with `` ` `` -> a line in a code block.


