### 0.5.3

* Update syntect dependency to 1.0.4.

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
