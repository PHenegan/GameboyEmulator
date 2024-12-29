#[derive(Debug, PartialEq, Eq)]
struct PpuError;

trait Ppu {
   // TODO - finish 
}

/// # Tile
/// Represents a Tile (8x8 square of pixels in the PPU's render frame
struct Tile {
    x: u8,
    y: u8,
    // TODO - finish
    // (Figure out how to make this in a way that doesn't get messed up with Gameboy Color
    // support)
}

/// # DmgPpu
/// Represents the Pixel Processing Unit of an original Gameboy system (i.e. non-color)
struct DmgPpu {
    // TODO - finish
}

