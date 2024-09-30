// Print hello


main:
  // set index to where we want to store the sprite
  ld i 0x300

  // load the sprite into memory
  ld v0 0x20
  ld v1 0x70
  ld v2 0x20

  // load sprite into memory
  ld i v2

  // xy cords
  ld v3 0x0a
  ld v4 0x08

  drw v3 v4 0x3

loop:
  wkp v5
  jmp loop
