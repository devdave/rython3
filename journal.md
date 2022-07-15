Rython
=======

Name: Rython Gen 3    
Author: David J. Ward  
summary: A "basic" Python virtual machine implemented in Rust
Reason/purpose: Improve both Rust and Python skills

## General Notes

Rython is a toy project and not intended for production environments.   

The final/master goal is to not only implement Python's unit test module but to also
attempt to pass as many of the unit tests as is possible.

## Tokenizer notes

At the bottom is a IO Buffer lines iterator/vector which outputs foundational tokens:  

* Name
* OP
* Number
* NEWLINE
* INDENT
* DEDENT
* EOF/ENDMARKER
* COMMENT

More than one foundational token will exist on the same line therefore the lowest tier
tokenizer will return a `vec<Token>`

Next tier will process OP tokens and given them a TokenType value.

Because the bottom tier handles indent/dedent, it will need to have a Token State.


## Tokenizer line processor design/scratch pad

Processor's main entry point is ::consume(lines: vec[str])-> Result<vec[Token], ProcessorError>

internally consume will instantiate a new Self instance. 

::consume iterates over lines, passing each to transform_line(line: &str) -> vec[Token] which is unrolled and
appended to a master list/vector.


## Processor design

I really need a whiteboard for this.   My brute force/naive attempt to handle
indent/dedent hasn't worked correctly.  It does indents correctly but not dedents.

What is need is a state machine:  

* While/if in whitespace look to see if line ends on a comment
* While/if in string (" or """) consume everything until (" or """)
* While/if in paren/bracket... maybe consume everything?


## Processor identation

Step 1 is to verify that tabs and spaces are not mixed together.

Step 2 is to check the current indent in stack.   

* if stack len == 0 and WS len is not 0.  push to stack and return indent


* If stack len == 0 or stack element < than current WS, append new WS len, return indent
* If stack element > than current WS, pop elements until current WS == element, return dedent 
 for each != pop.
* If stack len == 0 and WS == 0, do nothing

# Processor - managed line helper struct

I decided to manage lines