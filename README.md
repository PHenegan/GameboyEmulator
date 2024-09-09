# Game Boy Emulator

This project is an emulator for original Game Boy systems (codenamed DMG), and potentially
Game Boy Color (codenamed CGB) in the future.

# Motivations and Goals

**NOTE: This project is not intended to enable piracy of any kind. My goals for this emulator
are entirely educational.**

My previous emulator/interpreter project was a CHIP-8 interpreter written in C with the SDL library
and CMake for a build system. There are a lot of things I learned from that project, and my goals
for this project are extensions to some of those things.

I want to learn more CPU design. As a virtual system, the CHIP-8 had a very simple memory layout
and a very small number of opcodes. In addition to that, there was no dedicated chip for handling
graphics (like the Game Boy's PPU). Part of my goal for this project is learning about more
advanced ways to handle memory chips, as the Game Boy has several techniques for using more memory
than the CPU can actually address (some Game Boy games were as big as 2 MiB despite the fact that
the Game Boy memory addresses are only 16 bits long. The CHIP-8 also lacks any form of
interrupts, which contrasts against the several that the Game Boy has.

I also relied heavily on a single source of information for the CHIP-8, which documented all of
the opcodes, there behaviors, and even gave advice on reimplimenting that in code. For this
emulator, I'd like to look at more technical documentation on the system by myself.

Lastly, I want to learn about graphics rendering libraries such as OpenGL and Vulkan. After using
some basic SDL calls for rendering, I'm interested in learning more about the next step below
that, which would be using a rendering library directly.

I see a Game Boy emulator as a good step to learning all of these things. In the future,
I may also do a Game Boy Advance emulator to learn about more modern ISAs (ARM32) and CPU
emulation as well.

# What Works? What's Left?
**This mostly an empty repo right now. Nothing works, everything is left. At this point, 
this list is really just a way for me to itemize the order in which I want to try to do 
everything.**
- See the GitHub Issues page for more specific details on progress
- There is a basic skeleton for the CPU registers, as well as the functions needed for memory
  access by the CPU. See the memory/mod.rs file.
- There is a trait for handling reads/writes to cartridge memory. *It is not necessarily finalized
  because I haven't added save support*
  - Implementations exist for unmapped cartridges, MBC1, MBC2, and MBC3 cartridges. 
    At some point I may try to implement the others but they are lower priority since I want to
    target basic functionality first.
  - The constructors currently implemented are filler and will likely be removed or made private.
    This is because I wanted to wait until I have the logic for several controllers implemented
    before making the logic that actually loads ROM and RAM data into them.
- I have some idea from watching a technical video, but I'm unclear on where the PPU fits into
  the program's structure. I'll likely do more research into that as well

# Building and Running
WIP

# Useful Resources

[This video](https://www.youtube.com/watch?v=HyzD8pNlpwI) which is a great technical dive into
the Game Boy system as a whole. I also included some typed notes I took on this video
for the purpose of internalizing the system better. They can be found in the `extra/` folder in this
repository.

[https://gbdev.io/gb-opcodes//optables/](https://gbdev.io/gb-opcodes//optables/) is a great website
which has some documentation on the Gameboy's opcodes

[The Pan Docs](https://gbdev.io/pandocs/) Another great resource for technical documentation 
on various different parts of the Game Boy and Game Boy Color
