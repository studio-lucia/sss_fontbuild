// Each pixel is two bits; there are four pixels in each byte.
// This provides mappings between the colour value and its definition in bits.
pub const BITS_TRANSPARENT : [u8; 2] = [1, 1];
pub const BITS_GREY        : [u8; 2] = [0, 1];
pub const BITS_DARK_BLUE   : [u8; 2] = [0, 0];
pub const BITS_LIGHT_BLUE  : [u8; 2] = [1, 0];

// Some things need to differentiate between the two games' SYSTEM.DAT files
pub enum Game {
    SSS,
    SSSC,
}

pub const SSS_SYSTEM_DAT_SIZE        : u64 = 252232;
pub const SSSC_SYSTEM_DAT_SIZE       : u64 = 258552;
pub const SSS_FONT_START             : u64 = 0x16FC4;
pub const SSS_FONT_LEN_UNCOMPRESSED  : u64 = 29376;
pub const SSS_FONT_LEN_COMPRESSED    : u64 = 28097;
pub const SSSC_FONT_START            : u64 = 0x1757C;
pub const SSSC_FONT_LEN_UNCOMPRESSED : u64 = 32256;
pub const SSSC_FONT_LEN_COMPRESSED   : u64 = 30885;
