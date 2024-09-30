// print out all opcodes -- not intended to be run on chip 8

// vf -- is used as a flag instruction -- particularly for overflow

org 0x2

opcodes:

  // add i vx - 0xfx1e - expected 0xF21E
  // i = i + vx -- does not affect the vf on overflow
  add i v2

  // add vx vy - 0x8xy4 - expected 0x8234 -- vy is not affected
  // vf is set to 1 if there is a carry (larger than 0xFF or 255)
  // vf is set to 0 if there is no carry
  add v2 v3

  // add vx nn - 0x7xnn -- expected 0x7201
  // vx = vx + nn -- does not affect the vf on overflow
  add v2 0x01

  // and vx vy - 0x8xy2 - expected 0x8232 -- vx = vx & vy -- vy is not affected
  and v2 v3

  // call addr - 0x2nnn - expected 0x2A3E -- call a subroutine at address
  // address needs to be even
  call 0xA3E

  // cls - 0x00E0 -- clear the screen
  cls

  // drw vx vy n - 0xDxyn - expected 0xd234 
  // draw sprite n pixels tall from memory location in register i
  // at position (vx, vy)
  drw v2 v3 0x4

  // jmp addr - 0x1nnn - expected 0x1A3E -- jump to address
  // address needs to be even
  jmp 0xA3E

  // jmp v0 nnn - 0xBnnn - expected 0xB33F -- jump to address + v0
  // address needs to be even
  jmp v0 0x33E

  // ld vx nn - 0x6xnn - expected 0x6223
  ld v2 0x23

  // ld vx vy - 0x8xy0 - expected 0x8320 -- load vy into vx
  ld v3 v2

  // ld i nnn - 0xAnnn - expected 0xA2AB -- load nnn into i
  ld i 0x2AB

  // ld vx dt - 0xFx07 - expected 0xF307 -- load delay timer into vx 
  ld v3 dt

  // ld dt vx - 0xFx15 - expected 0xF315 -- load vx into delay timer
  ld dt v3

  // ld st v2 - 0xFx18 - expected 0xF218 -- load vx into sound timer
  ld st v2

  // ld f vx - 0xFx29 - expected 0xF329 -- load sprite location for digit vx into i
  ld f v3

  // ld b vx - 0xFx33 - expected 0xF233 -- store bcd representation of vx in memory locations i, i+1, i+2
  ld b v2

  // ld i vx - 0xFx55 - expected 0xF255 -- store registers v0 through vx in memory starting at i
  ld i v2

  // ld vx i - 0xFx65 - expected 0xF265 -- load registers v0 through vx from memory starting at i
  ld v2 i

  // or vx vy - 0x8xy1 - expected 0x8231 -- vx = vx | vy -- vy is not affected
  or v2 v3

  // ret - 0x00EE -- return from a subroutine
  ret

  // rnd vx nn - 0xCxnn - expected 0xC423 -- vx = random number & nn
  rnd v4 0x23

  // se vx vy - 0x5xy0 - expected 0x5230 
  // skip next instruction if vx == vy
  se v2 v3

  // se vx nn - 0x3xnn - expected 0x3231
  // skip next instruction if vx == nn
  se v2 0x31

  // shl vx - 0x8xyE - expected 0x830E -- vx = vx << 1
  shl v3

  // shr vx - 0x8xy6 - expected 0x8306 -- vx = vx >> 1
  shr v3

  // sknp vx - 0xExA1 - expected 0xE2A1 
  // skip next instruction if key in vx is not pressed
  sknp v2

  // skp vx - 0xEx9E - expected 0xE29E 
  // skip next instruction if key in vx is pressed
  skp v2

  // sne vx vy - 0x9xy0 - expected 0x9230
  // skip next instruction if vx != vy
  sne v2 v3

  // sne vx nn - 0x4xnn - expected 0x4231
  sne v2 0x31

  // sub vx, vy - 0x8xy5 - expected 0x8235 -- vx = vx - vy 
  // if vx >= vy, vf = 1 else vf = 0
  sub v2 v3

  // subn vx, vy - 0x8xy7 - expected 0x8237 -- vx = vy - vx
  // if vy >= vx, vf = 1 else vf = 0
  subn v2 v3

  // wkp vx - 0xFx0A - expected 0xF30A -- wait for key press and store in vx
  wkp v3

  // xor vx vy - 0x8xy3 - expected 0x8233 -- vx = vx ^ vy -- vy is not affected
  xor v2 v3
