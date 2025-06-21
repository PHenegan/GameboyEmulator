pub const TILE_SIZE: usize = 16;
pub const TILE_DATA_SIZE: usize = 256;
pub const TILE_MAP_SIZE: usize = 32;

// Blocks of memory where Tile data is stored
pub const TILE_BLOCK_0: u16 = 0x8000;
pub const TILE_BLOCK_1: u16 = 0x8800;
pub const TILE_BLOCK_2: u16 = 0x9000;

// Blocks of memory where tile maps can be stored for the window or background
pub const TILE_MAP_0: u16 = 0x9800;
pub const TILE_MAP_1: u16 = 0x9BFF;

// OAM region where sprite data is stored
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OBJ_SIZE: usize = 40;

// LCD registers
pub const LCDC: u16 = 0xFF40; // LCD control
pub const LY: u16 = 0xFF44; // LCD Y coordinate
pub const LYC: u16 = 0xFF45; // LY Compare
pub const STAT: u16 = 0xFF41; // LCD Status

// Offsets of the background on the *canvas* (larger than the screen)
// "Scroll X" and "Scroll Y"
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;

// Window position on the screen
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;
