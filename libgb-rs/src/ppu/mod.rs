#[derive(Debug, PartialEq, Eq)]
struct PpuError;

trait Ppu {
   // TODO - finish 
}

/// # Tile
/// Represents a Tile (8x8 square of pixels) to be displayed on a Game Boy screen
///
/// Parameters:
/// - `rows`: the block of 8 pixel rows in each tile. Each row is 8 pixels wide.
///           the leftmost byte of each row contains the second bit in each pixel's color,
///           while the first byte contains the first bit. This is because of the little-endian
///           byte ordering.
struct Tile {
    pub rows: [u16; 8],
    x: u8, // these might be unnecessary but I'm putting them here for now
    y: u8,
    // (Figure out how to make this in a way that doesn't get messed up with Gameboy Color
    // support)
    // Q: Should Sprites be part of this struct? They can be 8x16 and I'm guessing they follow
    //    the same format in terms of bit layout?
}

/// # DmgPpu
/// Represents the Pixel Processing Unit of an original Gameboy system (i.e. non-color)
struct DmgPpu {
    tilemap: Vec<Vec<Tile>>,
    // objects - TODO
    // TODO - finish
}

