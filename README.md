[![Build Status](https://travis-ci.org/jmacdonald/scribe.svg?branch=master)](https://travis-ci.org/jmacdonald/scribe)

# Scribe: A text editor toolkit

Provides a layered set of types for dealing with text documents:

* Gap buffer - Data structure optimized for successive, close-proximity edits.
* Buffer - Wrapper that provides bounds-checked cursor management and file persistence.
* Workspace - Collection of buffers with selection management.

The buffer type also provides basic type detection and lexing, making use of the lexers provided by the [luthor](https://github.com/jmacdonald/luthor) library.
