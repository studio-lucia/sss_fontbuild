// Each pixel is two bits; there are four pixels in each byte.
// This provides mappings between the colour value and its definition in bits.
pub const BITS_TRANSPARENT : [u8; 2] = [1, 1];
pub const BITS_GREY        : [u8; 2] = [0, 1];
pub const BITS_DARK_BLUE   : [u8; 2] = [0, 0];
pub const BITS_LIGHT_BLUE  : [u8; 2] = [1, 0];
