https://www.youtube.com/watch?v=HyzD8pNlpwI

# Introduction/Overview

Most of this is specifically about the original Game Boy (DMG), not the Game Boy Color (CGB)
- However, I think most of the actual CPU logic is going to be the same between DMG and CGB - the only notable differences (I think) will be the memory amount, clock frequency, and the GPU portion (along with the screen)

**Some Specs**
- 1 MHz* 8-bit CPU
	- Asterisk because a lot of people consider the CPU to be 4MHz
- 8 KB of RAM
- 8 KB of VRAM
- 160 x 144 resolution
- 4 different colors
- 10 "sprites" per line (sprites explained further down in the PPU section)

**What is in the SoC?**
- CPU - Sharp LR35902
- Interrupt Controller
- Timer
- Memory
- Boot ROM
- Joypad input controller
- Serial Data transfer
- Sound Controller
- PPU (Pixel Processing Unit)
	- Handles the graphics
# CPU

**Sharp LR35902**
- weird in-between of an Intel 8080 and a Zilog Z80
	-  Note - Zilog Z80 was is an expansion on the 8080, meaning it could do everything an 8080 could do
	- It has the same core features as the 8080 (but actually not all of them), but some extras that were part of the Z80
	- Also has some of its own extensions

8 x 8-bit registers
- 'A' - accumulator register - handles arithmetic and logic
- 'F' - flag register - first 4 bits are flags, first and last are the most useful
	- "Zero and carry are the two useful ones, the other two are for decimal adjusts" - the 33c3 guy
	- 'Z' - zero flag
	- 'N'
	- 'H'
	- 'C' - carry flag
- 'B', 'C', 'D', 'E', 'H', 'L' - general purpose registers that can hold data
	- Note - they can also be combined into 16-bit registers in pairs
		- 'BC,' 'DE,' 'HL'
- (hl) can also be used - this is the 8-bit value at the memory address in 'HL'

At around 12 minutes-ish he starts talking about Interrupts. I don't really understand what interrupts are at the moment so I'll try to come back to this later

From 11:48 until around __ he talks about the instructions. Since I'm going to be looking at documentation for this anyway I'm kind of just listening to this.
## Clocks

The reason the CPU is either 4 MHz or 1 MHz is because of the LD instruction
- It takes 16 clocks at 4 MHz or 4 clocks at 1 MHz. All of the other instructions are divisible by 4, and the CPU is bottlenecked by the memory anyway, so either can be used just as well
- Using 1 MHz for comparison makes it easier to compare to other systems at the time

CPU - 4 MiHz
RAM - 1 MiHz
PPU - 4 MiHz
VRAM - 2 MiHz
- Technically they are all in "Mibihertz," which uses base 2 (i.e. 1024 * 1024 = 1 MiHz)

Cycle ~ machine cycle @ 1 MiHz

# Memory (~17:05)

16-bits can be used for an address in memory, meaning the CPU can only "see" 16 KiB of addresses
- ROM - first 32Kib, starting from `0x0` -> `0x7FFF` (includes a boot ROM)
- VRAM - 8 KiB, `0x8000`->`9FFF`
- External RAM (from cartridge) - 8 KiB, `0xA000`->`0xBFFF`
- Internal RAM - 8 KiB, `0xC000`->`0xDFFF`
- Reserved Area not really being used for much - `0xE000`->`0xFFFF`
	- There is a bit here though
	- OAM RAM (described as special purpose VRAM) - `0xFE00` -> `FEFF`
	- IO data for peripherals - `0xFF00` -> `0xFF7F`
	- HRAM - `0xFF80` -> `0xFFFF`

Although the Gameboy can only see 32 KiB of the cartridge ROM at a time, some games had controllers for intercepting the gameboy and splitting the cartridge into different "Banks"
- "Bank 0" is always the first 16 KiB of the ROM
- "Bank 1-n" is represented by the second 16 KiB of the ROM, and gets switched out by sending "magic values" which get intercepted by the cartridge controller in order to then switch the memory address
	- The same model can be used to map extra external RAM into the gameboy

The Boot ROM actually compares the first part of the ROM with the logo to make sure the game actually has the logo - if it doesn't it won't boot the game
- This allowed Nintendo to control which games came out on the system - including the logo in a cartridge without Nintendo's permission would be a copyright and trademark violation
- The screen does not get cleared after the logo sequence - a lot of games took advantage of this to do something cool with the Nintendo logo

The Boot ROM is programmed into the system and takes up the first few addresses of memory, but will turn itself off (giving those addresses back to the cartridge) after the sequence completes

From here on out talking about registers generally refers to an address in memory reserved for specific uses
# Joypad Input (~22:10)

contains 6 GPIO pins - P10 - P14

DPad is 1 column, buttons are the other
4 rows (4 dpad buttons, 4 other buttons)

# Serial (~22:30)
I'm mostly going to ignore the link cable stuff for now, I may come back to it at some point but idk

# Timer (~23:10)
1 timer - put start value into "TMA" modulo register, select a clock speed (see the table), and then the timer will go until the number overflows
- Can optionally generate an interrupt

# Interrupts (~23:40)

Again, I don't really understand how these work but my guess is that it pauses the execution somehow and does something else (maybe like a JAL but event-based?)

Can be generated by a number of things:
- Joypad
- Serial
- Timer
- LCD STAT (used by PPU)
- V-Blank (used by PPU)

Uses 2 registers:
- `0xFFFF` - Interrupt Enable (IE) - whether or not you want to have an interrupt for specific things
- `0xFF0F` - Interrupt Flag (IF) - whether or not each interrupt is still pending (Note - they will all jump to different addresses, so you can use that to determine which interrupt occurred)

# Sound Controller (~24:10)

4 Channels ("Voices"), 5 registers each
- registers for Control, Frequency, Volume, Length, Sweep ()
- 2 Pulse channels, 1 wave channel, 1 noise channel (pseudo-random noise)

I'm not going to go too in-depth here because sound is one of my lower priorities
(and again, I don't really understand it)

# PPU (Pixel Processing Unit) (~29:15)

Note - a lot of this seems specific to the DMG, would likely be different to some extent for CGB

Odd part - it can't actually address individual pixels, only 8 x 8 tiles at a time
- Looking at it this way, it would have 20 x 18 tiled display

For every line of pixels you need 2 bytes
- Each byte represents one of the bits for every row

For example, a row of pixels whose values are all `01` would be stored as `00 FF` in hex
- 16 bytes will describe an entire tile

The values are not necessarily linked to a particular color - although there are only 4 colors, the palette is arbitrary (this allows for easy inverting of colors by changing the palette)

## Viewport

There is a 32x32 "viewport" which the system can scroll through as a player moves around. This is kind of like a camera that can follow the player
- The screen will wrap around the viewport, so extra tiles can be loaded in while an area is not in the camera. This is what many games do to have an "infinite" map size

## Window

Another image which is drawn over the viewport. This can be used for static things like menus and HUDs.

Can be drawn at any position on the screen

# Sprites
Images which don't fit in an 8x8 tile system - like an NPC, enemies, moving characters, etc.

Each sprite has some attributes
- Position (X and Y)
- Tile Number (256 possible tiles in the system)
- Priority (determines whether it gets drawn on top of other things)
	- Priority of 1 will only get drawn on top of white pixels
	- Priority of 0 will get drawn on top of everything
	- Sprites with lower id numbers will be drawn on top of sprites with higher id numbers
- Flip X
- Flip Y
- Palette - because there is transparency, you can have 2 palettes on sprites

Notes about positioning
- They are drawn from the bottom right corner upwards (so kind of opposite of what you'd expect)

You can only have 40 sprites on the screen at a time, and 10 per line

Sprite attributes are stored in OAM RAM
- OAM likely stands for OBJ Access Memory because the sprites are technically OBJ's even though that term doesn't really mean that anymore

technically there is a 5th color - LCD off (lol)
- This turns the entire screen off

## Memory Map (~39:18)

Breakdown of storage requirements
- 4 KiB for sprite tiles
- 4 KiB for background tiles
- 1 KiB for the background map
- 1 KiB for the window map

This adds up to more than the 8 KiB of VRAM. How do programmers work around this?
- These "zones" can overlap. Ultimately the addresses are arbitrary, but oftentimes some of the memory will be shared between sprite and background tiles, meaning anything in the overlap can be used as both a sprite and a background tile.

## Drawing (~40:40)

Drawing is done from the top down, from left to right

A lot of this drawing logic is based on specific timing, I'll have to rewatch this part again in the future because this is kind of going over my head right now

The video kind of ends after this because this section is like 10-15 minutes long