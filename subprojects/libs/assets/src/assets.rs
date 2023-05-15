use platform_types::{GFX_LENGTH, ARGB};

/*
    A way to convert an image to an array of bytes:
    Given an image called `image.png` use the following imagemagick command:
    ```
    magick .\image.png -define h:format=rgba -depth 8 -size 128x128  image.h
    ```
    (Note that as of this writing, outputting into argb is not supported.)

    Then use regular expression find-and-replace to convert the array to the format you want.
    For example, you might replace `0x22, 0x22, 0x22, 0xFF,` with `index6,`, then similarly
    replace the rest of the colours with something containing their index value, then remove
    all instances of `index`, leaving just the indices. Format further as needed.

    Some possiby useful regex replaces (quotes delimit the given regex):
    Replace "0x([0-9A-F][0-9A-F]), 0x([0-9A-F][0-9A-F]), 0x([0-9A-F][0-9A-F]), 0x([0-9A-F][0-9A-F]), "
    with "0x\4\1\2\3, "
    Note that this swaps the byte order from rgba to argb.
*/

pub const GFX: [ARGB; GFX_LENGTH] = include!("gfx.in");