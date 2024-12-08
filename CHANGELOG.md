### 0.8.6

* Updated `Workspace::update_current_syntax` method to match against buffer file
  names when an extension isn't present, enabling support for formats like
  `Makefile`. Since this method is called by `Workspace::open_buffer` and
  `Workspace::add_buffer`, these methods will now correctly apply syntax
  definitions when available.

### 0.8.5

* Add new `Workspace::buffer_paths` method to list all buffer paths in the
  workspace.
* Add new `Workspace::current_buffer_index` method, which when combined with the
  `buffer_paths` method above, can be used to highlight the current buffer
  filename and/or path.

### 0.8.4

* Rename `Buffer::replace_content` method to just `replace`
* Re-implement `replace` as a proper operation, allowing it to be undone/redone
* Fix an issue from `0.8.3` where `Buffer::reload` would leave buffer flagged as modified

### 0.8.3

* Add new `Buffer::replace_content` method to replace all content in-place
* Updated `Buffer::reload` to raise an error when path is missing
* Updated `GapBuffer::new` to accept both `String` and `&str`

### 0.8.2

* Update Buffer `reload` method to preserve cursor position or line when
  possible.
* Replace GapBuffer `to_string` method with `fmt::Display` trait implementation.

### 0.8.1

* Removed unused luthor dependency
* Upgrade unicode-segmentation dependency
* Use semver-based dependency format

### 0.8.0

Lots of changes, some of which break public APIs. Almost all of these are
related to a syntect upgrade, which jumps _three_ major versions, from v2 to v5.

#### Removed methods

* Buffer::tokens(&self)
* Buffer::current_scope(&self)
* Workspace::current_buffer(&mut self)
* Workspace::contains_buffer_with_path(&self, &Path) -> bool

#### New methods

* Buffer::file_extension(&self) -> Option<String>
* Workspace::current_buffer_tokens(&'a self) -> Result<TokenSet<'a>>

#### Changed methods

* TokenSet::new(String, &'a SyntaxDefinition) -> TokenSet<'a>
  * is now TokenSet::new(String, &'a SyntaxReference, &'a SyntaxSet) -> TokenSet<'a>
* TokenSet::iter(&self) -> TokenIterator
  * is now TokenSet::iter(&self) -> Result<TokenIterator>
* TokenIterator::new(&'a str, &'a SyntaxDefinition) -> TokenIterator<'a>
  * is now TokenIterator::new(&'a str, &'a SyntaxReference, &'a SyntaxSet) -> Result<TokenIterator<'a>>
* Workspace::new(&Path) -> Result<Workspace>
  * is now Workspace::new(&Path, Option<&Path>) -> Result<Workspace> to allow loading user syntax definitions during construction

#### Buffer tokenization

Tokenization previously only required a syntax definition, but now requires a
SyntaxSet too. As a result, buffers are no longer able to produce a TokenSet
using only their own fields. Given this added dependency, setting up
tokenization is now the responsibility of Workspace::current_buffer_tokens.

#### Current workspace buffer

Accessing the workspace's current buffer was previously done using
Workspace::current_buffer(&mut self). Despite the encapsulation benefits this
accessor approach provides, Rust isn't smart enough to infer that we're only
borrowing one field, and the entire workspace is borrowed for the given
lifetime. This was incompatible with the new tokenization requirements, which
use both the syntax set from the workspace and the definition from the buffer;
calling current_buffer() on the workspace would make accessing its syntax_set
impossible.

To solve this, the current buffer is now accessed directly as a field.

#### Error propagation

Syntect now returns proper errors instead of panicking. To make proper use of
these, the TokenIterator type has been updated to handle Result-based returns.
When an error is discovered, iteration is halted, and the error is stored in a
public field on the iterator. This allows consumers to check for errors after
iterating, rather than on every returned value during iteration.

### 0.7.2

* Renamed Distance type's `from_str` method to `of_str`, to prevent ambiguity
  with the standard library FromStr trait method.

### 0.7.1

* Add `update_current_syntax` method to the `Workspace` type, to re-attempt
  assigning a syntax definition to the current buffer. This is most useful when
  a buffer path changes, and the new path can provide a more appropriate
  definition.

### 0.7.0

* Add `change_callback` field to the `Buffer` type, to facilitate cache
  invalidation implementations. When specified, this closure will be called
  whenever buffer data changes.

### 0.6.3

* Updated syntect dependency to 2.1.0.

### 0.6.2

* Expose the LineIterator type, which includes trailing newlines. This is
  required when using the `syntect` crate to highlight a buffer, as its line
  parser expects these.

### 0.6.1

* Updated documentation links and README.

### 0.6.0

* Implemented the Add and AddAssign operation traits for Position.
  * The latter replaces the previous self-mutating add method.
* Derived Copy and Clone traits for Distance type.
* Dropped mutable borrow for buffer search.

### 0.5.9

* Fixed an issue with `TokenIterator` where position offsets were byte-based
  rather than grapheme cluster-based, which is the contract/convention
  applied throughout the rest of the library.

### 0.5.8

* Expose the `Workspace` type's `syntax_set` field. This allows augmenting the
  default syntax definition set with additional entries.

### 0.5.7

* Updated syntect dependency to 1.7.1.
* Added a new current_scope method to the Buffer type, which will return the
  scope stack at the cursor position.
* Updated the Buffer tokens method to return a MissingSyntaxDefinition error
  when a syntax hasn't been configured for the buffer. This will provide more
  context to consumers when we're unable to return a TokenSet.

### 0.5.6

* Updated syntect dependency to 1.3.0.
* Fixed an issue with the token iterator where, under certain circumstances,
  trailing newline characters would be included in a lexeme.

### 0.5.4

* Updated workspace current_buffer_path method to return the buffer
  path as-is when it cannot be represented relative to the workspace.
* Updated dependency specifications to lock both major and minor versions.

### 0.5.3

* Updated syntect dependency to 1.0.4.

### 0.5.2

* Remove print call in insert operation reverse method (for shame)

### 0.5.1

* Updated gap buffer, cursor, and insert operation types to support grapheme clusters.
* Cleaned up buffer save method error handling.

### 0.5.0

* Migrated token lexers to syntect-based implementation.
  * The tokens method now yields a lazily-executed iterable type.
  * The Token type now contains a position and scope stack (rather than category).
* Updated Workspace and Buffer types to canonicalize their paths.
  * Added a current_buffer_path method to the Workspace type, which returns a path relative to the workspace.
* Replaced Option<io::Error> returns with proper Result types.
* Fixed an issue where reloading a buffer would not persist its ID.
* Added a Position constructor.

### 0.4.10

* Added Distance relative/vector type.
* Updated Position type to support adding Distances (via add method).

### 0.4.9

* Added reload method to buffer type.

### 0.4.8

* Fixed buffer doc tests for modified and line_count methods.
* Update buffer line_count method to handle trailing newlines.

### 0.4.7

* Removed caching from buffer type's `data` and `tokens` methods, removing
  mutability requirements.
* Added line_count method to buffer type.
* Updated insert operation to use Into trait rather than converting &str -> String.

### 0.4.6

* Added `id` field to buffer type.
* Updated workspace to populate buffer `id` field with a unique value when added
  to the workspace.

### 0.4.5

* Changed workspace open/close buffer methods to work relative to current buffer.

### 0.4.4

* Implement prefix matching without use of unstable pattern feature.

### 0.4.3

* Added modification tracking to buffers.

### 0.4.2

* Renamed start/end of buffer cursor movement methods.

### 0.4.1

* Add mutable dereferencing to buffer cursor type.

### 0.4.0

* Update all types to use type-scoped constructor idiom (e.g. Buffer::new() instead of buffer::new()).

### 0.3.10

* Add workspace open_buffer method to add a buffer by path, also guarding
  against adding a buffer with the same path as an existing one.

### 0.3.9

* Add cursor methods for moving to start and end of buffer.

### 0.3.8

* Update luthor dependency to v0.1.4.

### 0.3.7

* Fixed an issue where searching a buffer with multi-byte unicode characters
  would panic due to incorrect/non-byte-offset slicing.

### 0.3.1

* Fixed an issue where reversing an insert operation with a trailing newline
  character would remove more data than was inserted.
* Fixed various line length calculations incorrectly using byte lengths instead
  of character counts.

### 0.3.0

* Added a read method to Buffer type for reading arbitrary ranges.

### 0.2.7

* Removed is_valid method from Range type.
* Privatized Range type fields, added a constructor, as well as accessor methods.
  This is to guarantee that start and end fields are in the correct order.

### 0.2.6

* Fixed an issue where reading from a gap buffer when the gap is at the start
  of the requested range would return the content prefixed with the gap contents.
* Updated gap buffers to gracefully handle delete ranges that extend beyond a
  line's last column or the end of the document.

### 0.2.5

* Fixed target gap offset resolution introduced in v0.2.4, which did not handle
  all cases.

### 0.2.4

* Updated gap buffer to move gap to the end of the buffer before reallocating.
  This prevents a split/two-segment gap when inserting enough data at the start
  to trigger a reallocation.

### 0.2.3

* Drop empty operation groups when explicitly ended via end_operation_group or
  implicitly via undo.
