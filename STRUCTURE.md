A jpeg file has multiple segments, the APP1 segment usually contains exif data
The exif data is organized in IFD (Image File Directory) sections.
There is the basic IFD0 section, possible followed by the IFD1 section (an extension).
The IFD0 section also contains tags that are offsets to other sections if present:
The GPS, SubIFD and InteropIFD sections.

A section contains exif tags. An exif tag is of a certain format and can contain multiple components of that format (e.g. a float, or an int).
So we have 4 levels of iterators:

 - An iterator for all JPEG segments in the file (JPEGSegmentIterator)
 - An iterator to join all IFD sections in the APP1 segment (ExifTagIterator)
 - An iterator for all tags in an IFD section (SectionIterator)
 - An iterator for all components in a tag (ComponentIterator)

 SectionIterator and JPEGSegmentIterator are only used internally,
 ExifTagIterator and ComponentIterator are used by consumers of the library.

 The exif data also has a data section after all the IFDs. An EXIF tag is fixed in size (12 bytes) of which 4 bytes are for the value. If an EXIF value is bigger (like a string, or the thumbnail), the exif tag contains an offset to where the longer date is located in the data section behind the IFDs.