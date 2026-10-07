/* Scalar RV32IM/ILP32 output of CRYPTOGAMS riscv/poly1305-riscv.pl 32 */
/* Upstream: dot-asm/cryptogams db452f400063973daff7c2a03337e585677c698d */
/* Generated with clang -target riscv32 -march=rv32im -mabi=ilp32 -E -P. */
/* License: LICENSE.cryptogams and LICENSE.openssl. */

.option pic
.text
.globl poly1305_init
.type poly1305_init,@function
poly1305_init:
.align 2
 sw x0,0(x10)
 sw x0,4(x10)
 sw x0,8(x10)
 sw x0,12(x10)
 sw x0,16(x10)
 beqz x11,.Lno_key
 andi x5,x11,3 # x11 % 4
 sub x11,x11,x5 # align x11
 sll x5,x5,3 # byte to bit offset
 lw x14,0(x11)
 lw x15,4(x11)
 lw x16,8(x11)
 lw x17,12(x11)
 beqz x5,.Laligned_key
 lw x7,16(x11)
 sub x6,x0,x5
 srl x14,x14,x5
 sll x28,x15,x6
 srl x15,x15,x5
 or x14,x14,x28
 sll x28,x16,x6
 srl x16,x16,x5
 or x15,x15,x28
 sll x28,x17,x6
 srl x17,x17,x5
 or x16,x16,x28
 sll x7,x7,x6
 or x17,x17,x7
.Laligned_key:
 lui x5,0x10000
 addi x5,x5,-1 # 0x0fffffff
 and x14,x14,x5
 addi x5,x5,-3 # 0x0ffffffc
 and x15,x15,x5
 and x16,x16,x5
 and x17,x17,x5
 sw x14,20(x10)
 sw x15,24(x10)
 sw x16,28(x10)
 sw x17,32(x10)
 srl x6,x15,2
 srl x7,x16,2
 srl x28,x17,2
 add x15,x15,x6 # s1 = r1 + (r1 >> 2)
 add x16,x16,x7
 add x17,x17,x28
 sw x15,36(x10)
 sw x16,40(x10)
 sw x17,44(x10)
.Lno_key:
 li x10,0
 ret
.size poly1305_init,.-poly1305_init
.globl poly1305_blocks
.type poly1305_blocks,@function
poly1305_blocks:
.align 2
 andi x12,x12,-16 # complete blocks only
 beqz x12,.Labort
 add x2,x2,-4*12
 sw x1, 4*11(x2)
 sw x8, 4*10(x2)
 sw x9, 4*9(x2)
 sw x18, 4*8(x2)
 sw x19, 4*7(x2)
 sw x20, 4*6(x2)
 sw x21, 4*5(x2)
 sw x22, 4*4(x2)
 sw x23, 4*3(x2)
 sw x24, 4*2(x2)
 andi x1,x11,3
 andi x11,x11,-4 # align x11
 slli x1,x1,3 # byte to bit offset
 lw x8,0(x10) # load hash value
 lw x9,4(x10)
 lw x18,8(x10)
 lw x19,12(x10)
 lw x20,16(x10)
 lw x21,20(x10) # load key
 lw x22,24(x10)
 lw x23,28(x10)
 lw x24,32(x10)
 lw x5,36(x10)
 lw x6,40(x10)
 lw x7,44(x10)
 add x12,x12,x11 # end of buffer
.Loop:
 lw x14,0(x11) # load input
 lw x15,4(x11)
 lw x16,8(x11)
 lw x17,12(x11)
 beqz x1,.Laligned_inp
 lw x29,16(x11)
 sub x30,x0,x1
 srl x14,x14,x1
 sll x28,x15,x30
 srl x15,x15,x1
 or x14,x14,x28
 sll x28,x16,x30
 srl x16,x16,x1
 or x15,x15,x28
 sll x28,x17,x30
 srl x17,x17,x1
 or x16,x16,x28
 sll x29,x29,x30
 or x17,x17,x29
.Laligned_inp:
 srli x28,x20,2 # modulo-scheduled reduction
 andi x29,x20,-4
 andi x20,x20,3
 add x14,x14,x8 # accumulate input
  add x29,x29,x28
 sltu x8,x14,x8
 add x14,x14,x29 # ... and residue
 sltu x29,x14,x29
 add x15,x15,x9
  add x8,x8,x29 # carry
 sltu x9,x15,x9
 add x15,x15,x8
 sltu x8,x15,x8
 add x16,x16,x18
  add x9,x9,x8 # carry
 sltu x18,x16,x18
 add x16,x16,x9
 sltu x9,x16,x9
 add x17,x17,x19
  add x18,x18,x9 # carry
 sltu x19,x17,x19
 add x17,x17,x18
 mulhu x9,x21,x14; mul x8,x21,x14 # d0*r0
  sltu x18,x17,x18
  add x19,x19,x18 # carry
 mulhu x29,x7,x15; mul x28,x7,x15 # d1*s3
  add x20,x20,x13
  add x11,x11,16
  add x20,x20,x19
 mulhu x31,x6,x16; mul x13,x6,x16 # d2*s2
  add x8,x8,x28
  add x9,x9,x29
  sltu x28,x8,x28
  add x9,x9,x28
 mulhu x29,x5,x17; mul x28,x5,x17 # d3*s1
  add x8,x8,x13
  add x9,x9,x31
  sltu x13,x8,x13
  add x9,x9,x13
 mulhu x18,x22,x14; mul x13,x22,x14 # d0*r1
  add x8,x8,x28
  add x9,x9,x29
  sltu x28,x8,x28
  add x9,x9,x28
 mulhu x29,x21,x15; mul x28,x21,x15 # d1*r0
  add x9,x9,x13
  sltu x13,x9,x13
  add x18,x18,x13
 mulhu x31,x7,x16; mul x13,x7,x16 # d2*s3
  add x9,x9,x28
  add x18,x18,x29
  sltu x28,x9,x28
  add x18,x18,x28
 mulhu x29,x6,x17; mul x28,x6,x17 # d3*s2
  add x9,x9,x13
  add x18,x18,x31
  sltu x13,x9,x13
  add x18,x18,x13
 mul x13,x5,x20 # h4*s1
  add x9,x9,x28
  add x18,x18,x29
  sltu x28,x9,x28
  add x18,x18,x28
 mulhu x19,x23,x14; mul x28,x23,x14 # d0*r2
  add x9,x9,x13
  sltu x13,x9,x13
  add x18,x18,x13
 mulhu x31,x22,x15; mul x13,x22,x15 # d1*r1
  add x18,x18,x28
  sltu x28,x18,x28
  add x19,x19,x28
 mulhu x29,x21,x16; mul x28,x21,x16 # d2*r0
  add x18,x18,x13
  add x19,x19,x31
  sltu x13,x18,x13
  add x19,x19,x13
 mulhu x31,x7,x17; mul x13,x7,x17 # d3*s3
  add x18,x18,x28
  add x19,x19,x29
  sltu x28,x18,x28
  add x19,x19,x28
 mul x28,x6,x20 # h4*s2
  add x18,x18,x13
  add x19,x19,x31
  sltu x13,x18,x13
  add x19,x19,x13
 mulhu x31,x24,x14; mul x13,x24,x14 # d0*r3
  add x18,x18,x28
  sltu x28,x18,x28
  add x19,x19,x28
 mulhu x29,x23,x15; mul x28,x23,x15 # d1*r2
  add x19,x19,x13
  sltu x13,x19,x13
  add x31,x31,x13
 mulhu x13,x21,x17; mul x17,x21,x17 # d3*r0
  add x19,x19,x28
  add x31,x31,x29
  sltu x28,x19,x28
  add x31,x31,x28
 mulhu x29,x22,x16; mul x28,x22,x16 # d2*r1
  add x19,x19,x17
  add x31,x31,x13
  sltu x17,x19,x17
  add x31,x31,x17
 mul x13,x7,x20 # h4*s3
  add x19,x19,x28
  add x31,x31,x29
  sltu x28,x19,x28
  add x31,x31,x28
 mul x20,x21,x20 # h4*r0
  add x19,x19,x13
  sltu x13,x19,x13
  add x31,x31,x13
 add x20,x31,x20
 li x13,1 # if we loop, padbit is 1
 bne x11,x12,.Loop
 sw x8,0(x10) # store hash value
 sw x9,4(x10)
 sw x18,8(x10)
 sw x19,12(x10)
 sw x20,16(x10)
 lw x1, 4*11(x2)
 lw x8, 4*10(x2)
 lw x9, 4*9(x2)
 lw x18, 4*8(x2)
 lw x19, 4*7(x2)
 lw x20, 4*6(x2)
 lw x21, 4*5(x2)
 lw x22, 4*4(x2)
 lw x23, 4*3(x2)
 lw x24, 4*2(x2)
 add x2,x2,4*12
.Labort:
 ret
.size poly1305_blocks,.-poly1305_blocks
.globl poly1305_emit
.type poly1305_emit,@function
.align 2
poly1305_emit:
 lw x13,16(x10)
 lw x5,0(x10)
 lw x6,4(x10)
 lw x7,8(x10)
 lw x28,12(x10)
 srli x10,x13,2 # final reduction
 andi x14,x13,-4
 andi x13,x13,3
 add x10,x10,x14
 add x5,x5,x10
 sltu x10,x5,x10
  addi x14,x5,5 # compare to modulus
 add x6,x6,x10
  sltiu x15,x14,5
 sltu x10,x6,x10
  add x15,x15,x6
 add x7,x7,x10
  sltu x16,x15,x6
 sltu x10,x7,x10
  add x16,x16,x7
 add x28,x28,x10
  sltu x17,x16,x7
 sltu x10,x28,x10
  add x17,x17,x28
 add x13,x13,x10
  sltu x10,x17,x28
  add x10,x10,x13
 srl x10,x10,2 # see if it carried/borrowed
 sub x10,x0,x10
 xor x14,x14,x5
 xor x15,x15,x6
 xor x16,x16,x7
 xor x17,x17,x28
 and x14,x14,x10
 and x15,x15,x10
 and x16,x16,x10
 and x17,x17,x10
 xor x14,x14,x5
 xor x15,x15,x6
 xor x16,x16,x7
 xor x17,x17,x28
 lw x5,0(x12) # load nonce
 lw x6,4(x12)
 lw x7,8(x12)
 lw x28,12(x12)
 add x14,x14,x5 # accumulate nonce
 sltu x10,x14,x5
 add x15,x15,x6
 sltu x6,x15,x6
 add x15,x15,x10
 sltu x10,x15,x10
 add x10,x10,x6
 add x16,x16,x7
 sltu x7,x16,x7
 add x16,x16,x10
 sltu x10,x16,x10
 add x10,x10,x7
 add x17,x17,x28
 add x17,x17,x10
 srl x5,x14,8 # write mac value
 srl x6,x14,16
 srl x7,x14,24
 sb x14, 0(x11)
 sb x5,1(x11)
 srl x5,x15,8
 sb x6,2(x11)
 srl x6,x15,16
 sb x7,3(x11)
 srl x7,x15,24
 sb x15, 4(x11)
 sb x5,5(x11)
 srl x5,x16,8
 sb x6,6(x11)
 srl x6,x16,16
 sb x7,7(x11)
 srl x7,x16,24
 sb x16, 8(x11)
 sb x5,9(x11)
 srl x5,x17,8
 sb x6,10(x11)
 srl x6,x17,16
 sb x7,11(x11)
 srl x7,x17,24
 sb x17, 12(x11)
 sb x5,13(x11)
 sb x6,14(x11)
 sb x7,15(x11)
 ret
.size poly1305_emit,.-poly1305_emit
.string "Poly1305 for RISC-V, CRYPTOGAMS by @dot-asm"
