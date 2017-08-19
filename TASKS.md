 - [x] Make tests run again
 - [x] Think about nicer API for skipping ahead in a cursor/make cursor immutable
 - [x] Decide whether to pass Cursor values or refs: as value!
 - [x] Write unit test for EXIF value parsing
 - [x] Write code to iterate over a single IFD section
 - [x] Write unit tests for a single IFD section
 - [_] Write code to iterate over all IFD sections
 	mostly done, just need to handle tags that describe offsets to other IFDs
 - [ ] Give every container an `iter()` method instead of directly containing
        iterators. This way we don't need to be able to mutate the tag just to
        iterate them. This is also a guideline for rust libraries. This is
        a problem detected in the task above, because reading the tag for getting
        the offset gives the same problem. You'd either have to own and move the
        tag back and forth, or borrow the tag mutubly, but then you couldn't iterate
        it a second time.
 - [ ] Write unit tests for all IFD sections
 - [ ] add step to map RawExifTag -> ExifTag
 	tag_type as enum with tag names
 	Simplify EXIF values
 	Flatten ComponentIterator
 - [ ] write code that combines jpeg and exif parsing, to be the api exported
 - [ ] Ignore APP1 sections with wrong exif header, don't produce error
 - [ ] Better naming, rename a bunch of stuff. Ideas:
 	- move TIFF stuff into own module directory
 	- IFD should be called directories, not sections
 - [ ] See if we can use the byteorder crate instead off our own unsafe code
 - [ ] Introduce trait for string decoding, allow injection if user needs encoding detection
 - [ ] Add support for MakerNotes and raw files.
 	I think they also use TIFF format, with different header values.
