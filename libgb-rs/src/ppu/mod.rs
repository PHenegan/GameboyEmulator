use std::array::from_fn;

use crate::GameBoySystem;
use crate::constants::display;

#[derive(Debug, PartialEq, Eq)]
pub struct PpuError;

pub struct Pixel {
    /// Value between [0, 3]
    pub color: u8,
    /// Value between [0, 7]
    pub palette: u8,
    pub sprite_priority: bool,
    pub lcd_priority: bool,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            color: 0,
            palette: 0,
            sprite_priority: false,
            lcd_priority: false,
        }
    }
}

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
#[derive(Default)]
pub struct Sprite {
    /// position on the x-axis of the screen to render the sprite
    x: u8,
    /// position on the y-axis of the screen to render the sprite
    y: u8,
    /// the index of the sprite in the tilesheet which contains its image data
    tile_num: u8,
    /// whether or not the sprite should be drawn on top of the background.
    /// If false, the sprite will only be drawn on top of colors with a pallete ID of 0
    priority: bool,
    /// whether the sprite should be horizontally flipped
    x_flip: bool,
    /// whether the sprite should be vertically flipped
    y_flip: bool,
    /// whether to use the palette specified by OBP0 or OBP1 in non-CGB mode
    palette_dmg: bool,
    // These are commented out because they are specific to the Game Boy Color
    // palette_cgb: u8,
    // bank: u8,
}

impl From<[u8; 4]> for Sprite {
    fn from(bytes: [u8; 4]) -> Self {
        Sprite {
            x: bytes[0],
            y: bytes[1],
            tile_num: bytes[2],
            priority: bytes[3] & 0x80 != 0,
            x_flip: bytes[3] & 0x20 != 0,
            y_flip: bytes[3] & 0x40 != 0,
            palette_dmg: bytes[3] & 0x8 != 0,
        }
    }
}

/// # LcdFlags
/// A convenience struct to separate the LCD control register into its separate flags
struct LcdFlags {
    /// sets whether the LCD is receiving power 
    enabled: bool,
    /// if true use 0x9C00-0x9FFF for window tile map, otherwise use 0x9800-0x9BFF
    window_map_high: bool,
    /// sets whether the window map should be drawn over the screen
    window_enabled: bool,
    tile_data_high: bool,
    // If true use 0x9C00-0x9FFF for background map, otherwise use 0x9800-0x9BFF
    background_map_high: bool,
    /// use 8x16 sprites if true, 8x8 sprites otherwise
    tall_sprites: bool,
    /// sets whether sprites should be drawn
    enable_sprites: bool,
    /// If in CGB mode (game boy color), setting this bit will draw all objects on top of
    /// the background and window. If in DMG mode, the background and window will not be displayed
    priority: bool,
}

impl From<u8> for LcdFlags {
    fn from(value: u8) -> Self {
        LcdFlags {
            enabled: value & 0x80 != 0,
            window_map_high: value & 0x40 != 0,
            window_enabled: value & 0x20 != 0,
            tile_data_high: value & 0x10 != 0,
            background_map_high: value & 0x8 != 0,
            tall_sprites: value & 0x4 != 0,
            enable_sprites: value & 0x2 != 0,
            priority: value & 0x1 != 0,
        }
    }
}

/// # Tile
/// represents a single 8x8 block of pixels on the Game Boy's screen
type Tile = [u8; display::TILE_SIZE];

/// # Tile
/// represents a mapping of a position on the screen to the index of a tile in the tile data. 
type TileMap = [[u8; display::TILE_MAP_SIZE]; display::TILE_MAP_SIZE];

impl GameBoySystem {
    /// # ppu_draw
    /// A single step in the Game Boy's PPU (Pixel Processing Unit) drawing pipeline
    pub fn ppu_draw(&mut self) -> Result<(), PpuError> {
        let sprites = self.sprite_data()?;
        let sprite_map = self.sprite_maps()?;

        let lcd_control: LcdFlags = self.memory.load_byte(display::LCDC)
            .ok_or(PpuError)?
            .into();
        let tile_data = self.tile_data(lcd_control.tile_data_high)?;
        let background_map = self.tile_map(lcd_control.background_map_high)?;
        let window_map = self.tile_map(lcd_control.window_map_high)?;
        todo!("Finish the pixel pipeline");
    }

    /// # sprite_maps
    /// Create a list of sprite maps from the Game Boy's OAM memory
    pub fn sprite_maps(&self) -> Result<[Sprite; display::OBJ_SIZE], PpuError> {

        // using default here because the load_byte function might return an error
        let mut sprites: [Sprite; display::OBJ_SIZE] = from_fn(|_idx| Sprite::default());
        for idx in 0..display::OBJ_SIZE {
            // Load the sprite data from the 4 bytes in OAM memory
            let address = display::OAM_START + (4 * idx) as u16;
            let mut entry_bytes: [u8; 4] = [0; 4];
            for byte_idx in 0..4 {
                entry_bytes[0] = self.memory.load_byte(address + byte_idx)
                    .ok_or(PpuError)?;
            };
            sprites[idx] = Sprite::from(entry_bytes);
        }

        return Ok(sprites);
    }

    /// # tile_map
    /// Create a map storing a set of indices of tiles to draw at specific points on the map
    ///
    /// ## Parameters:
    /// - `high_map`: whether to use memory region 0x9800-0x9BFF (true) or 0x9C00-0x9FFF (false)
    ///
    /// ## Returns:
    /// return an 32 x 32 grid of tile indices, or a PpuError if the tiles cannot be loaded
    pub fn tile_map(&self, high_map: bool) -> Result<TileMap, PpuError> {
        let mut tile_map = [[0; display::TILE_MAP_SIZE]; display::TILE_MAP_SIZE];
        let start = if high_map {
            display::TILE_MAP_1
        } else {
            display::TILE_MAP_0
        };

        for y in 0..display::TILE_MAP_SIZE {
            for x in 0..display::TILE_MAP_SIZE {
                let address = start + (y * display::TILE_MAP_SIZE + x) as u16;
                tile_map[y][x] = self.memory.load_byte(address)
                    .ok_or(PpuError)?;
            }
        }

        return Ok(tile_map);
    }

    /// # sprite_data
    /// load the list of sprite tiles from the PPU's memory block
    pub fn sprite_data(&self) -> Result<[Tile; display::TILE_DATA_SIZE], PpuError> {
        self.tile_data(false)
    }

    /// # tile_data
    /// load a list of tiles from the PPU's memory block
    /// 
    /// ## Parameters:
    /// - `signed_index`: whether or not to treat the tile data as a signed index.
    ///                   This will load from different memory regions depending on the value
    /// 
    /// ## Returns:
    /// The function returns an array of 256 "tiles," which themselves are 16 bytes of encoded
    /// palette data to create an 8x8 tile
    pub fn tile_data(
        &self, signed_index: bool
    ) -> Result<[Tile; display::TILE_DATA_SIZE], PpuError> {
        let mut tiles = [[0; display::TILE_SIZE]; display::TILE_DATA_SIZE];
        
        // load the data by halves in order to avoid using signed logic for indices
        let (first, second) = if signed_index {
            (display::TILE_BLOCK_2, display::TILE_BLOCK_1)
        } else {
            (display::TILE_BLOCK_0, display::TILE_BLOCK_1)
        };

        self.load_tile_block(&mut tiles, first, 0)?;
        self.load_tile_block(&mut tiles, second, display::TILE_DATA_SIZE/2)?;
        Ok(tiles)
    }

    fn load_tile_block(
        &self, tiles: &mut [Tile; display::TILE_DATA_SIZE], block_address: u16, idx: usize
    ) -> Result<(), PpuError> {
        for block_idx in 0..display::TILE_DATA_SIZE/2 {
            for tile_idx in 0..display::TILE_SIZE {
                let address = block_address + (block_idx as u16) * 16 + (tile_idx as u16);
                tiles[block_idx + idx][tile_idx] = self.memory.load_byte(address)
                    .ok_or(PpuError)?;
            }
        }
        Ok(())
    }
}
