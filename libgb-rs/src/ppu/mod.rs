use crate::{constants::display::{TILE_BLOCK_0, TILE_BLOCK_1, TILE_BLOCK_2}, GameBoySystem};

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

pub struct DmgPpu {
    window_x: u8,
    window_y: u8,
    scroll_x: u8,
    scroll_y: u8,
    signed_tile_index: bool,
    object_map: Vec<Tile>,
}

/// # Tile
/// represents a single 8x8 block of pixels on the Game Boy's screen
type Tile = [u8; 16];

impl GameBoySystem {

    /// # ppu_draw
    /// A single step in the Game Boy's PPU (Pixel Processing Unit) drawing pipeline
    pub fn ppu_draw(&mut self) -> Result<(), PpuError> {
        let sprite_data = self.sprite_map()?;
        todo!()
    }

    /// # sprites
    /// Create a list of sprites from the Game Boy's OAM memory
    pub fn sprites(&self) -> Result<Vec<Sprite>, PpuError> {

        todo!()
    }

    /// # sprite_map
    /// load the list of tiles from the PPU's memory block
    pub fn sprite_map(&self) -> Result<[Tile; 256], PpuError> {
        self.tilemap(false)
    }

    pub fn tilemap(&self, signed_index: bool) -> Result<[Tile; 256], PpuError> {
        let mut tiles = [[0; 16]; 256];
        
        // load the data by halves in order to avoid using signed logic for indices
        let (first, second) = if signed_index {
            (TILE_BLOCK_2, TILE_BLOCK_1)
        } else {
            (TILE_BLOCK_0, TILE_BLOCK_1)
        };

        self.load_tile_block(&mut tiles, first, 0)?;
        self.load_tile_block(&mut tiles, second, 128)?;
        Ok(tiles)
    }

    fn load_tile_block(
        &self, tiles: &mut [Tile; 256], block_address: u16, idx: usize
    ) -> Result<(), PpuError> {
        for block_idx in 0..128 {
            for tile_idx in 0..16 {
                let address = block_address + (block_idx as u16) * 16 + (tile_idx as u16);
                tiles[block_idx + idx][tile_idx] = self.memory.load_byte(address)
                    .ok_or(PpuError)?;
            }
        }
        Ok(())
    }
}
