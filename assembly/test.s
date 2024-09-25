// my test file
// numbers are in hex
org 0x200

start:
  jmp start
  jmp 0x33f
  jmp v0 0x3f
  rnd vf 0x56
  rnd v2 0x45
  call start
  ret
