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
