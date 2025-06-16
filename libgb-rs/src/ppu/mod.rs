#[derive(Debug, PartialEq, Eq)]
pub struct PpuError;

// Idea - just have the PPU logic be in the form of a "draw" function on the GameBoySystem struct
// - construct the image by assembling Tiles, Sprites, and Windows
// - Use a "render" function as a method on the image to produce a flat image to send
//   to the frontend
// - Maybe have some sort of system framebuffer to render over representing the screen state,
//   so that way a lot of the timing can be done
//     - for now this could be the only state preserved but there might be room for optimization
//       there in the future
//
// Potential Problems
// - See game boy talk notes - some distortion effects rely on timing, I may have to add some sort
//   of persistant state between function calls to get this to work properly
// - OAM ram should not be accessed during specific states - "OAM search" and "Pixel Transfer"
//   periods (~48 minutes into the ultimate game boy talk video)


/// # Tile
/// Represents a Tile (8x8 square of pixels) to be displayed on a Game Boy screen
///
/// ## Fields:
/// - `rows`: the block of 8 pixel rows in each tile. Each row is 8 pixels wide.
///           the leftmost byte of each row contains the second bit in each pixel's color,
///           while the first byte contains the first bit. This is because of the little-endian
///           byte ordering.
pub struct Tile {
    x: u8, // these might be unnecessary but I'm putting them here for now
    y: u8,
    tile_num: u8,
    // (Figure out how to make this in a way that doesn't get messed up with Gameboy Color
    // support)
}

/// # Sprite
/// Represents a sprite ("Object" by Nintendo's terms) which is drawn at an arbitrary position on
/// the game boy screen. Examples of sprites include players, projectiles, enemies, etc. In
/// particular, this object represents an OAM entry in the OAM region of RAM - takes up 4 bytes
///
/// # LIMITATIONS
/// - there can only be 40 sprites on the screen at any point in time
/// - there can only be 10 sprites on a given line at any point in time
///     - any more, and their data should be skipped *on that line*
///     - decided by the 11th item in the list of sprites - order determined by the program
///
/// ## Fields:
/// - `x`: position on the x-axis of the screen to render the sprite
/// - `y`: position on the y-axis of the screen to render the sprite
/// - `tile_num`: the index of the sprite in the tilesheet which contains its image data
/// - `priority`: whether or not the sprite should be drawn on top of the background.
///               If false, the sprite will only be drawn on top of colors with a pallete ID of 0
pub struct Sprite {
    x: u8,
    y: u8,
    tile_num: u8,
    // if priority is 0, only draw the sprite on top of background pixels w/ pallette id 0
    // otherwise, draw on top of all background
    priority: bool,
    flip_x: bool,
    flip_y: bool,
    pallete: u8, // takes up 5 bits?
}

/// # Window
/// Represents the "Window" tiles used by the Game Boy's PPU. These can be drawn on top of the
/// background (and sprites?) as a separate map. This is primarily used for menus, HUDs, etc.
pub struct Window {
    x: u8,
    y: u8,
    address: u16,
}

