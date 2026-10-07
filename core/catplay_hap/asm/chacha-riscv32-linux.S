/* Scalar RV32IM/ILP32 output of CRYPTOGAMS riscv/chacha-riscv.pl */
/* Upstream: dot-asm/cryptogams db452f400063973daff7c2a03337e585677c698d */
/* Generated with clang -target riscv32 -march=rv32im -mabi=ilp32 -E -P. */
/* CatPlay addition: inp == NULL writes raw keystream without input loads. */
/* License: LICENSE.cryptogams and LICENSE.openssl. */

.text
.option pic
.type __ChaCha,@function
.align 2
__ChaCha:
 lw x16, 4*0(x2)
 lw x17, 4*1(x2)
 lw x18, 4*2(x2)
 lw x19, 4*3(x2)
 lw x20, 4*4(x2)
 lw x21, 4*5(x2)
 lw x22, 4*6(x2)
 lw x23, 4*7(x2)
 lw x24, 4*8(x2)
 lw x25, 4*9(x2)
 lw x26,4*10(x2)
 lw x27,4*11(x2)
 mv x28,x15
 lw x29,4*13(x2)
 lw x30,4*14(x2)
 lw x31,4*15(x2)
.Loop:
 addi x14,x14,-1
 add x16,x16,x20 # Q0
  add x17,x17,x21 # Q1
   add x18,x18,x22 # Q2
    add x19,x19,x23 # Q3
 xor x28,x28,x16
  xor x29,x29,x17
   xor x30,x30,x18
    xor x31,x31,x19
 srl x5,x28,16
  srl x6,x29,16
   srl x7,x30,16
    srl x8,x31,16
 sll x28,x28,16
  sll x29,x29,16
   sll x30,x30,16
    sll x31,x31,16
 or x28,x28,x5
  or x29,x29,x6
   or x30,x30,x7
    or x31,x31,x8
 add x24,x24,x28
  add x25,x25,x29
   add x26,x26,x30
    add x27,x27,x31
 xor x20,x20,x24
  xor x21,x21,x25
   xor x22,x22,x26
    xor x23,x23,x27
 srl x5,x20,20
  srl x6,x21,20
   srl x7,x22,20
    srl x8,x23,20
 sll x20,x20,12
  sll x21,x21,12
   sll x22,x22,12
    sll x23,x23,12
 or x20,x20,x5
  or x21,x21,x6
   or x22,x22,x7
    or x23,x23,x8
 add x16,x16,x20
  add x17,x17,x21
   add x18,x18,x22
    add x19,x19,x23
 xor x28,x28,x16
  xor x29,x29,x17
   xor x30,x30,x18
    xor x31,x31,x19
 srl x5,x28,24
  srl x6,x29,24
   srl x7,x30,24
    srl x8,x31,24
 sll x28,x28,8
  sll x29,x29,8
   sll x30,x30,8
    sll x31,x31,8
 or x28,x28,x5
  or x29,x29,x6
   or x30,x30,x7
    or x31,x31,x8
 add x24,x24,x28
  add x25,x25,x29
   add x26,x26,x30
    add x27,x27,x31
 xor x20,x20,x24
  xor x21,x21,x25
   xor x22,x22,x26
    xor x23,x23,x27
 srl x5,x20,25
  srl x6,x21,25
   srl x7,x22,25
    srl x8,x23,25
 sll x20,x20,7
  sll x21,x21,7
   sll x22,x22,7
    sll x23,x23,7
 or x20,x20,x5
  or x21,x21,x6
   or x22,x22,x7
    or x23,x23,x8
 add x16,x16,x21 # Q0
  add x17,x17,x22 # Q1
   add x18,x18,x23 # Q2
    add x19,x19,x20 # Q3
 xor x31,x31,x16
  xor x28,x28,x17
   xor x29,x29,x18
    xor x30,x30,x19
 srl x5,x31,16
  srl x6,x28,16
   srl x7,x29,16
    srl x8,x30,16
 sll x31,x31,16
  sll x28,x28,16
   sll x29,x29,16
    sll x30,x30,16
 or x31,x31,x5
  or x28,x28,x6
   or x29,x29,x7
    or x30,x30,x8
 add x26,x26,x31
  add x27,x27,x28
   add x24,x24,x29
    add x25,x25,x30
 xor x21,x21,x26
  xor x22,x22,x27
   xor x23,x23,x24
    xor x20,x20,x25
 srl x5,x21,20
  srl x6,x22,20
   srl x7,x23,20
    srl x8,x20,20
 sll x21,x21,12
  sll x22,x22,12
   sll x23,x23,12
    sll x20,x20,12
 or x21,x21,x5
  or x22,x22,x6
   or x23,x23,x7
    or x20,x20,x8
 add x16,x16,x21
  add x17,x17,x22
   add x18,x18,x23
    add x19,x19,x20
 xor x31,x31,x16
  xor x28,x28,x17
   xor x29,x29,x18
    xor x30,x30,x19
 srl x5,x31,24
  srl x6,x28,24
   srl x7,x29,24
    srl x8,x30,24
 sll x31,x31,8
  sll x28,x28,8
   sll x29,x29,8
    sll x30,x30,8
 or x31,x31,x5
  or x28,x28,x6
   or x29,x29,x7
    or x30,x30,x8
 add x26,x26,x31
  add x27,x27,x28
   add x24,x24,x29
    add x25,x25,x30
 xor x21,x21,x26
  xor x22,x22,x27
   xor x23,x23,x24
    xor x20,x20,x25
 srl x5,x21,25
  srl x6,x22,25
   srl x7,x23,25
    srl x8,x20,25
 sll x21,x21,7
  sll x22,x22,7
   sll x23,x23,7
    sll x20,x20,7
 or x21,x21,x5
  or x22,x22,x6
   or x23,x23,x7
    or x20,x20,x8
 bnez x14,.Loop
 lw x5, 4*0(x2)
 lw x6, 4*1(x2)
 lw x7, 4*2(x2)
 lw x8, 4*3(x2)
 add x16,x16,x5
 lw x5,4*4(x2)
 add x17,x17,x6
 lw x6,4*5(x2)
 add x18,x18,x7
 lw x7,4*6(x2)
 add x19,x19,x8
 lw x8,4*7(x2)
 add x20,x20,x5
 lw x5,4*8(x2)
 add x21,x21,x6
 lw x6, 4*9(x2)
 add x22,x22,x7
 lw x7,4*10(x2)
 add x23,x23,x8
 lw x8,4*11(x2)
 add x24,x24,x5
 #lw x5,4*12(x2)
 add x25,x25,x6
 lw x6,4*13(x2)
 add x26,x26,x7
 lw x7,4*14(x2)
 add x27,x27,x8
 lw x8,4*15(x2)
 add x28,x28,x15
 add x29,x29,x6
 add x30,x30,x7
 add x31,x31,x8
 ret
.size __ChaCha,.-__ChaCha
.globl ChaCha20_ctr32
.type ChaCha20_ctr32,@function
.align 2
ChaCha20_ctr32:
 beqz x12,.Lzero_length
 add x2,x2,-(128+16*4)
 sw x1, ((128+16*4)-1*4)(x2)
 sw x8, ((128+16*4)-2*4)(x2)
.Lscalar_shortcut:
 sw x9, ((128+16*4)-3*4)(x2)
 sw x18, ((128+16*4)-4*4)(x2)
 sw x19, ((128+16*4)-5*4)(x2)
 sw x20, ((128+16*4)-6*4)(x2)
 sw x21, ((128+16*4)-7*4)(x2)
 sw x22, ((128+16*4)-8*4)(x2)
 sw x23, ((128+16*4)-9*4)(x2)
 sw x24, ((128+16*4)-10*4)(x2)
 sw x25, ((128+16*4)-11*4)(x2)
 sw x26,((128+16*4)-12*4)(x2)
 sw x27,((128+16*4)-13*4)(x2)
 lui x16,0x61707+1 # compose sigma
 lui x17,0x33206
 lui x18,0x79622+1
 lui x19,0x6b206
 addi x16,x16,-0x79b
 addi x17,x17,0x46e
 addi x18,x18,-0x2ce
 addi x19,x19,0x574
 lw x20, 4*0(x13)
 lw x21, 4*1(x13)
 lw x22, 4*2(x13)
 lw x23, 4*3(x13)
 lw x24, 4*4(x13)
 lw x25, 4*5(x13)
 lw x26,4*6(x13)
 lw x27,4*7(x13)
 lw x28,4*0(x14)
 lw x29,4*1(x14)
 lw x30,4*2(x14)
 lw x31,4*3(x14)
 sw x16, 4*0(x2)
 sw x17, 4*1(x2)
 sw x18, 4*2(x2)
 sw x19, 4*3(x2)
 sw x20, 4*4(x2)
 sw x21, 4*5(x2)
 sw x22, 4*6(x2)
 sw x23, 4*7(x2)
 sw x24, 4*8(x2)
 sw x25, 4*9(x2)
 sw x26,4*10(x2)
 sw x27,4*11(x2)
 mv x15,x28
 sw x29,4*13(x2)
 sw x30,4*14(x2)
 sw x31,4*15(x2)
 li x14,10
 jal .Loop
 beqz x11,.Lkeystream
 sltiu x14,x12,64
 or x1,x11,x10
 andi x1,x1,3 # both are aligned?
 bnez x14,.Ltail
 beqz x1,.Loop_aligned
.Loop_misaligned:
 lb x5,0(x11)
 lb x6,1(x11)
 srl x9,x16,8
 lb x7,2(x11)
 srl x13,x16,16
 lb x8,3(x11)
 srl x14,x16,24
 xor x16,x16,x5
 lb x5,4+0(x11)
 xor x9,x9,x6
 lb x6,4+1(x11)
 xor x13,x13,x7
 lb x7,4+2(x11)
 xor x14,x14,x8
 lb x8,4+3(x11)
 sb x16,0+0(x10)
 sb x9,0+1(x10)
 srl x9,x17,8
 sb x13,0+2(x10)
 srl x13,x17,16
 sb x14,0+3(x10)
 srl x14,x17,24
 xor x17,x17,x5
 lb x5,8+0(x11)
 xor x9,x9,x6
 lb x6,8+1(x11)
 xor x13,x13,x7
 lb x7,8+2(x11)
 xor x14,x14,x8
 lb x8,8+3(x11)
 sb x17,4+0(x10)
 sb x9,4+1(x10)
 srl x9,x18,8
 sb x13,4+2(x10)
 srl x13,x18,16
 sb x14,4+3(x10)
 srl x14,x18,24
 xor x18,x18,x5
 lb x5,12+0(x11)
 xor x9,x9,x6
 lb x6,12+1(x11)
 xor x13,x13,x7
 lb x7,12+2(x11)
 xor x14,x14,x8
 lb x8,12+3(x11)
 sb x18,8+0(x10)
 sb x9,8+1(x10)
 srl x9,x19,8
 sb x13,8+2(x10)
 srl x13,x19,16
 sb x14,8+3(x10)
 srl x14,x19,24
 xor x19,x19,x5
 lb x5,16+0(x11)
 xor x9,x9,x6
 lb x6,16+1(x11)
 xor x13,x13,x7
 lb x7,16+2(x11)
 xor x14,x14,x8
 lb x8,16+3(x11)
 sb x19,12+0(x10)
 sb x9,12+1(x10)
 srl x9,x20,8
 sb x13,12+2(x10)
 srl x13,x20,16
 sb x14,12+3(x10)
 srl x14,x20,24
 xor x20,x20,x5
 lb x5,20+0(x11)
 xor x9,x9,x6
 lb x6,20+1(x11)
 xor x13,x13,x7
 lb x7,20+2(x11)
 xor x14,x14,x8
 lb x8,20+3(x11)
 sb x20,16+0(x10)
 sb x9,16+1(x10)
 srl x9,x21,8
 sb x13,16+2(x10)
 srl x13,x21,16
 sb x14,16+3(x10)
 srl x14,x21,24
 xor x21,x21,x5
 lb x5,24+0(x11)
 xor x9,x9,x6
 lb x6,24+1(x11)
 xor x13,x13,x7
 lb x7,24+2(x11)
 xor x14,x14,x8
 lb x8,24+3(x11)
 sb x21,20+0(x10)
 sb x9,20+1(x10)
 srl x9,x22,8
 sb x13,20+2(x10)
 srl x13,x22,16
 sb x14,20+3(x10)
 srl x14,x22,24
 xor x22,x22,x5
 lb x5,28+0(x11)
 xor x9,x9,x6
 lb x6,28+1(x11)
 xor x13,x13,x7
 lb x7,28+2(x11)
 xor x14,x14,x8
 lb x8,28+3(x11)
 sb x22,24+0(x10)
 sb x9,24+1(x10)
 srl x9,x23,8
 sb x13,24+2(x10)
 srl x13,x23,16
 sb x14,24+3(x10)
 srl x14,x23,24
 xor x23,x23,x5
 lb x5,32+0(x11)
 xor x9,x9,x6
 lb x6,32+1(x11)
 xor x13,x13,x7
 lb x7,32+2(x11)
 xor x14,x14,x8
 lb x8,32+3(x11)
 sb x23,28+0(x10)
 sb x9,28+1(x10)
 srl x9,x24,8
 sb x13,28+2(x10)
 srl x13,x24,16
 sb x14,28+3(x10)
 srl x14,x24,24
 xor x24,x24,x5
 lb x5,36+0(x11)
 xor x9,x9,x6
 lb x6,36+1(x11)
 xor x13,x13,x7
 lb x7,36+2(x11)
 xor x14,x14,x8
 lb x8,36+3(x11)
 sb x24,32+0(x10)
 sb x9,32+1(x10)
 srl x9,x25,8
 sb x13,32+2(x10)
 srl x13,x25,16
 sb x14,32+3(x10)
 srl x14,x25,24
 xor x25,x25,x5
 lb x5,40+0(x11)
 xor x9,x9,x6
 lb x6,40+1(x11)
 xor x13,x13,x7
 lb x7,40+2(x11)
 xor x14,x14,x8
 lb x8,40+3(x11)
 sb x25,36+0(x10)
 sb x9,36+1(x10)
 srl x9,x26,8
 sb x13,36+2(x10)
 srl x13,x26,16
 sb x14,36+3(x10)
 srl x14,x26,24
 xor x26,x26,x5
 lb x5,44+0(x11)
 xor x9,x9,x6
 lb x6,44+1(x11)
 xor x13,x13,x7
 lb x7,44+2(x11)
 xor x14,x14,x8
 lb x8,44+3(x11)
 sb x26,40+0(x10)
 sb x9,40+1(x10)
 srl x9,x27,8
 sb x13,40+2(x10)
 srl x13,x27,16
 sb x14,40+3(x10)
 srl x14,x27,24
 xor x27,x27,x5
 lb x5,48+0(x11)
 xor x9,x9,x6
 lb x6,48+1(x11)
 xor x13,x13,x7
 lb x7,48+2(x11)
 xor x14,x14,x8
 lb x8,48+3(x11)
 sb x27,44+0(x10)
 sb x9,44+1(x10)
 srl x9,x28,8
 sb x13,44+2(x10)
 srl x13,x28,16
 sb x14,44+3(x10)
 srl x14,x28,24
 xor x28,x28,x5
 lb x5,52+0(x11)
 xor x9,x9,x6
 lb x6,52+1(x11)
 xor x13,x13,x7
 lb x7,52+2(x11)
 xor x14,x14,x8
 lb x8,52+3(x11)
 sb x28,48+0(x10)
 sb x9,48+1(x10)
 srl x9,x29,8
 sb x13,48+2(x10)
 srl x13,x29,16
 sb x14,48+3(x10)
 srl x14,x29,24
 xor x29,x29,x5
 lb x5,56+0(x11)
 xor x9,x9,x6
 lb x6,56+1(x11)
 xor x13,x13,x7
 lb x7,56+2(x11)
 xor x14,x14,x8
 lb x8,56+3(x11)
 sb x29,52+0(x10)
 sb x9,52+1(x10)
 srl x9,x30,8
 sb x13,52+2(x10)
 srl x13,x30,16
 sb x14,52+3(x10)
 srl x14,x30,24
 xor x30,x30,x5
 lb x5,60+0(x11)
 xor x9,x9,x6
 lb x6,60+1(x11)
 xor x13,x13,x7
 lb x7,60+2(x11)
 xor x14,x14,x8
 lb x8,60+3(x11)
 sb x30,56+0(x10)
 sb x9,56+1(x10)
 srl x9,x31,8
 sb x13,56+2(x10)
 srl x13,x31,16
 sb x14,56+3(x10)
 srl x14,x31,24
 xor x31,x31,x5
 xor x9,x9,x6
 xor x13,x13,x7
 xor x14,x14,x8
 sb x31,60(x10)
 addi x15,x15,1 # next counter value
 sb x9,61(x10)
 addi x12,x12,-64
 sb x13,62(x10)
 add x11,x11,64
 sb x14,63(x10)
 add x10,x10,64
 beqz x12,.Ldone
 sltiu x9,x12,64
 li x14,10
 jal __ChaCha
 beqz x9,.Loop_misaligned
 j .Ltail
.Loop_aligned:
 lw x5,0(x11)
 lw x6,4(x11)
 lw x7,8(x11)
 lw x8,12(x11)
 xor x16,x16,x5
 lw x5,16+0(x11)
 xor x17,x17,x6
 lw x6,16+4(x11)
 xor x18,x18,x7
 lw x7,16+8(x11)
 xor x19,x19,x8
 lw x8,16+12(x11)
 sw x16,0+0(x10)
 sw x17,0+4(x10)
 sw x18,0+8(x10)
 sw x19,0+12(x10)
 xor x20,x20,x5
 lw x5,32+0(x11)
 xor x21,x21,x6
 lw x6,32+4(x11)
 xor x22,x22,x7
 lw x7,32+8(x11)
 xor x23,x23,x8
 lw x8,32+12(x11)
 sw x20,16+0(x10)
 sw x21,16+4(x10)
 sw x22,16+8(x10)
 sw x23,16+12(x10)
 xor x24,x24,x5
 lw x5,48+0(x11)
 xor x25,x25,x6
 lw x6,48+4(x11)
 xor x26,x26,x7
 lw x7,48+8(x11)
 xor x27,x27,x8
 lw x8,48+12(x11)
 sw x24,32+0(x10)
 sw x25,32+4(x10)
 sw x26,32+8(x10)
 sw x27,32+12(x10)
 xor x28,x28,x5
 xor x29,x29,x6
 xor x30,x30,x7
 xor x31,x31,x8
 sw x28,48(x10)
 addi x15,x15,1 # next counter value
 sw x29,52(x10)
 addi x12,x12,-64
 sw x30,56(x10)
 add x11,x11,64
 sw x31,60(x10)
 add x10,x10,64
 sltiu x9,x12,64
 beqz x12,.Ldone
 li x14,10
 jal __ChaCha
 beqz x9,.Loop_aligned
.Ltail:
 mv x1,x2
 sw x17, 4*1(x2)
 sw x18, 4*2(x2)
 sw x19, 4*3(x2)
 sw x20, 4*4(x2)
 sw x21, 4*5(x2)
 sw x22, 4*6(x2)
 sw x23, 4*7(x2)
 sw x24, 4*8(x2)
 sw x25, 4*9(x2)
 sw x26,4*10(x2)
 sw x27,4*11(x2)
 sw x28,4*12(x2)
 sw x29,4*13(x2)
 sw x30,4*14(x2)
 sw x31,4*15(x2)
.Loop_tail:
 sltiu x14,x12,4
 bnez x14,.Last_word
 add x1,x1,4
 lb x5,0(x11)
 lb x6,1(x11)
 lb x7,2(x11)
 addi x12,x12,-4
 lb x8,3(x11)
 add x11,x11,4
 xor x5,x5,x16
 srl x16,x16,8
 xor x6,x6,x16
 srl x16,x16,8
 xor x7,x7,x16
 srl x16,x16,8
 xor x8,x8,x16
 lw x16,0(x1)
 sb x5,0(x10)
 sb x6,1(x10)
 sb x7,2(x10)
 sb x8,3(x10)
 add x10,x10,4
 j .Loop_tail
.Last_word:
 beqz x12,.Ldone
 addi x12,x12,-1
 lb x5,0(x11)
 add x11,x11,1
 xor x5,x5,x16
 srl x16,x16,8
 sb x5,0(x10)
 add x10,x10,1
 j .Last_word

/* x16..x31 hold the completed ChaCha block. Preserve the original */
/* state at 0..63(x2) while staging this block at 64..127(x2). */
.Lkeystream:
 sltiu x5,x12,64
 bnez x5,.Lkeystream_staged
 andi x5,x10,3
 bnez x5,.Lkeystream_staged
 // A full aligned block can be written directly from registers.
 sw x16,0(x10)
 sw x17,4(x10)
 sw x18,8(x10)
 sw x19,12(x10)
 sw x20,16(x10)
 sw x21,20(x10)
 sw x22,24(x10)
 sw x23,28(x10)
 sw x24,32(x10)
 sw x25,36(x10)
 sw x26,40(x10)
 sw x27,44(x10)
 sw x28,48(x10)
 sw x29,52(x10)
 sw x30,56(x10)
 sw x31,60(x10)
 addi x10,x10,64
 addi x12,x12,-64
 beqz x12,.Ldone
 addi x15,x15,1
 li x14,10
 jal __ChaCha
 j .Lkeystream
.Lkeystream_staged:
 sw x16,64(x2)
 sw x17,68(x2)
 sw x18,72(x2)
 sw x19,76(x2)
 sw x20,80(x2)
 sw x21,84(x2)
 sw x22,88(x2)
 sw x23,92(x2)
 sw x24,96(x2)
 sw x25,100(x2)
 sw x26,104(x2)
 sw x27,108(x2)
 sw x28,112(x2)
 sw x29,116(x2)
 sw x30,120(x2)
 sw x31,124(x2)
 li x5,64
 bltu x12,x5,.Lkeystream_partial
 j .Lkeystream_copy
.Lkeystream_partial:
 mv x5,x12
.Lkeystream_copy:
 addi x6,x2,64
.Lkeystream_byte:
 lbu x7,0(x6)
 sb x7,0(x10)
 addi x6,x6,1
 addi x10,x10,1
 addi x5,x5,-1
 bnez x5,.Lkeystream_byte
 sltiu x5,x12,65
 bnez x5,.Ldone
 addi x12,x12,-64
 addi x15,x15,1
 li x14,10
 jal __ChaCha
 j .Lkeystream
.Ldone:
 lw x1, ((128+16*4)-1*4)(x2)
 lw x8, ((128+16*4)-2*4)(x2)
 lw x9, ((128+16*4)-3*4)(x2)
 lw x18, ((128+16*4)-4*4)(x2)
 lw x19, ((128+16*4)-5*4)(x2)
 lw x20, ((128+16*4)-6*4)(x2)
 lw x21, ((128+16*4)-7*4)(x2)
 lw x22, ((128+16*4)-8*4)(x2)
 lw x23, ((128+16*4)-9*4)(x2)
 lw x24, ((128+16*4)-10*4)(x2)
 lw x25, ((128+16*4)-11*4)(x2)
 lw x26,((128+16*4)-12*4)(x2)
 lw x27,((128+16*4)-13*4)(x2)
 add x2,x2,(128+16*4)
 ret
.Lzero_length:
 ret
.size ChaCha20_ctr32,.-ChaCha20_ctr32
.section .rodata
.align 3
.string "ChaCha20 for RISC-V, CRYPTOGAMS by @dot-asm"
