# if (defined(__mips_smartmips) || defined(_MIPS_ARCH_MIPS32R3) || \
      defined(_MIPS_ARCH_MIPS32R5) || defined(_MIPS_ARCH_MIPS32R6)) \
      && !defined(_MIPS_ARCH_MIPS32R2)
#  define _MIPS_ARCH_MIPS32R2
# endif

# if (defined(_MIPS_ARCH_MIPS64R3) || defined(_MIPS_ARCH_MIPS64R5) || \
      defined(_MIPS_ARCH_MIPS64R6)) \
      && !defined(_MIPS_ARCH_MIPS64R2)
#  define _MIPS_ARCH_MIPS64R2
# endif

#if defined(__MIPSEB__) && !defined(MIPSEB)
# define MIPSEB
#endif

.text

.set	noat
.set	reorder

.align	5
.ent	__ChaCha
__ChaCha:
	.frame	$29,0,$31
	.mask	0,0
	.set	reorder
	lw		$10, 4*0($29)
	lw		$11, 4*1($29)
	lw		$12, 4*2($29)
	lw		$13, 4*3($29)
	lw		$14, 4*4($29)
	lw		$15, 4*5($29)
	lw		$16, 4*6($29)
	lw		$17, 4*7($29)
	lw		$18, 4*8($29)
	lw		$19, 4*9($29)
	lw		$20,4*10($29)
	lw		$21,4*11($29)
	move		$22,$30
	lw		$23,4*13($29)
	lw		$24,4*14($29)
	lw		$25,4*15($29)
.Lalt_entry:
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	move		$2,$10
	move		$3,$11
	move		$7,$12
	move		$8,$13
#endif
.Loop:
	addu		$10,$10,$14		# Q0
	 addu		$11,$11,$15		# Q1
	  addu		$12,$12,$16		# Q2
	   addu		$13,$13,$17		# Q3
	xor		$22,$22,$10
	 xor		$23,$23,$11
	  xor		$24,$24,$12
	   xor		$25,$25,$13
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$22,$22,16
	 rotr		$23,$23,16
	  rotr		$24,$24,16
	   rotr		$25,$25,16
#else
	srl		$2,$22,16
	 srl		$3,$23,16
	  srl		$7,$24,16
	   srl		$8,$25,16
	sll		$22,$22,16
	 sll		$23,$23,16
	  sll		$24,$24,16
	   sll		$25,$25,16
	or		$22,$22,$2
	 or		$23,$23,$3
	  or		$24,$24,$7
	   or		$25,$25,$8
#endif

	addu		$18,$18,$22
	 addu		$19,$19,$23
	  addu		$20,$20,$24
	   addu		$21,$21,$25
	xor		$14,$14,$18
	 xor		$15,$15,$19
	  xor		$16,$16,$20
	   xor		$17,$17,$21
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$14,$14,20
	 rotr		$15,$15,20
	  rotr		$16,$16,20
	   rotr		$17,$17,20
#else
	srl		$2,$14,20
	 srl		$3,$15,20
	  srl		$7,$16,20
	   srl		$8,$17,20
	sll		$14,$14,12
	 sll		$15,$15,12
	  sll		$16,$16,12
	   sll		$17,$17,12
	or		$14,$14,$2
	 or		$15,$15,$3
	  or		$16,$16,$7
	   or		$17,$17,$8
#endif

	addu		$10,$10,$14
	 addu		$11,$11,$15
	  addu		$12,$12,$16
	   addu		$13,$13,$17
	xor		$22,$22,$10
	 xor		$23,$23,$11
	  xor		$24,$24,$12
	   xor		$25,$25,$13
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$22,$22,24
	 rotr		$23,$23,24
	  rotr		$24,$24,24
	   rotr		$25,$25,24
#else
	srl		$2,$22,24
	 srl		$3,$23,24
	  srl		$7,$24,24
	   srl		$8,$25,24
	sll		$22,$22,8
	 sll		$23,$23,8
	  sll		$24,$24,8
	   sll		$25,$25,8
	or		$22,$22,$2
	 or		$23,$23,$3
	  or		$24,$24,$7
	   or		$25,$25,$8
#endif

	addu		$18,$18,$22
	 addu		$19,$19,$23
	  addu		$20,$20,$24
	   addu		$21,$21,$25
	xor		$14,$14,$18
	 xor		$15,$15,$19
	  xor		$16,$16,$20
	   xor		$17,$17,$21
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$14,$14,25
	 rotr		$15,$15,25
	  rotr		$16,$16,25
	   rotr		$17,$17,25
#else
	srl		$2,$14,25
	 srl		$3,$15,25
	  srl		$7,$16,25
	   srl		$8,$17,25
	sll		$14,$14,7
	 sll		$15,$15,7
	  sll		$16,$16,7
	   sll		$17,$17,7
	or		$14,$14,$2
	 or		$15,$15,$3
	  or		$16,$16,$7
	   or		$17,$17,$8
#endif
	addu		$10,$10,$15		# Q0
	 addu		$11,$11,$16		# Q1
	  addu		$12,$12,$17		# Q2
	   addu		$13,$13,$14		# Q3
	xor		$25,$25,$10
	 xor		$22,$22,$11
	  xor		$23,$23,$12
	   xor		$24,$24,$13
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$25,$25,16
	 rotr		$22,$22,16
	  rotr		$23,$23,16
	   rotr		$24,$24,16
#else
	srl		$2,$25,16
	 srl		$3,$22,16
	  srl		$7,$23,16
	   srl		$8,$24,16
	sll		$25,$25,16
	 sll		$22,$22,16
	  sll		$23,$23,16
	   sll		$24,$24,16
	or		$25,$25,$2
	 or		$22,$22,$3
	  or		$23,$23,$7
	   or		$24,$24,$8
#endif

	addu		$20,$20,$25
	 addu		$21,$21,$22
	  addu		$18,$18,$23
	   addu		$19,$19,$24
	xor		$15,$15,$20
	 xor		$16,$16,$21
	  xor		$17,$17,$18
	   xor		$14,$14,$19
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$15,$15,20
	 rotr		$16,$16,20
	  rotr		$17,$17,20
	   rotr		$14,$14,20
#else
	srl		$2,$15,20
	 srl		$3,$16,20
	  srl		$7,$17,20
	   srl		$8,$14,20
	sll		$15,$15,12
	 sll		$16,$16,12
	  sll		$17,$17,12
	   sll		$14,$14,12
	or		$15,$15,$2
	 or		$16,$16,$3
	  or		$17,$17,$7
	   or		$14,$14,$8
#endif

	addu		$10,$10,$15
	 addu		$11,$11,$16
	  addu		$12,$12,$17
	   addu		$13,$13,$14
	xor		$25,$25,$10
	 xor		$22,$22,$11
	  xor		$23,$23,$12
	   xor		$24,$24,$13
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$25,$25,24
	 rotr		$22,$22,24
	  rotr		$23,$23,24
	   rotr		$24,$24,24
#else
	srl		$2,$25,24
	 srl		$3,$22,24
	  srl		$7,$23,24
	   srl		$8,$24,24
	sll		$25,$25,8
	 sll		$22,$22,8
	  sll		$23,$23,8
	   sll		$24,$24,8
	or		$25,$25,$2
	 or		$22,$22,$3
	  or		$23,$23,$7
	   or		$24,$24,$8
#endif

	addu		$20,$20,$25
	 addu		$21,$21,$22
	  addu		$18,$18,$23
	   addu		$19,$19,$24
	xor		$15,$15,$20
	 xor		$16,$16,$21
	  xor		$17,$17,$18
	   xor		$14,$14,$19
#if defined(_MIPS_ARCH_MIPS32R2) || defined(_MIPS_ARCH_MIPS64R2)
	rotr		$15,$15,25
	 rotr		$16,$16,25
	  rotr		$17,$17,25
	   rotr		$14,$14,25
#else
	srl		$2,$15,25
	 srl		$3,$16,25
	  srl		$7,$17,25
	   srl		$8,$14,25
	sll		$15,$15,7
	 sll		$16,$16,7
	  sll		$17,$17,7
	   sll		$14,$14,7
	or		$15,$15,$2
	 or		$16,$16,$3
	  or		$17,$17,$7
	   or		$14,$14,$8
#endif
	.set		noreorder
	bnez		$1,.Loop
	subu		$1,$1,1
	.set		reorder

#if !defined(_MIPS_ARCH_MIPS32R2) && !defined(_MIPS_ARCH_MIPS64R2)
	lw		$2, 4*0($29)
	lw		$3, 4*1($29)
	lw		$7, 4*2($29)
	lw		$8, 4*3($29)
#endif
	addu		$10,$10,$2
	lw		$2,4*4($29)
	addu		$11,$11,$3
	lw		$3,4*5($29)
	addu		$12,$12,$7
	lw		$7,4*6($29)
	addu		$13,$13,$8
	lw		$8,4*7($29)
	addu		$14,$14,$2
	lw		$2,4*8($29)
	addu		$15,$15,$3
	lw		$3, 4*9($29)
	addu		$16,$16,$7
	lw		$7,4*10($29)
	addu		$17,$17,$8
	lw		$8,4*11($29)
	addu		$18,$18,$2
	#lw		$2,4*12($29)
	addu		$19,$19,$3
	lw		$3,4*13($29)
	addu		$20,$20,$7
	lw		$7,4*14($29)
	addu		$21,$21,$8
	lw		$8,4*15($29)
	addu		$22,$22,$30
	addu		$23,$23,$3
	addu		$24,$24,$7
	addu		$25,$25,$8
	jr		$31
.end	__ChaCha

.globl	ChaCha20_ctr32
.align	5
.ent	ChaCha20_ctr32
ChaCha20_ctr32:
	.frame	$29,128,$31
	.mask	0xc0ff0000,-4
	.set	noreorder
	subu	$29,$29,128
	sw		$31, (128-1*4)($29)
	sw		$30, (128-2*4)($29)
	sw		$23,(128-3*4)($29)
	sw		$22,(128-4*4)($29)
	sw		$21, (128-5*4)($29)
	sw		$20, (128-6*4)($29)
	sw		$19, (128-7*4)($29)
	sw		$18, (128-8*4)($29)
	sw		$17, (128-9*4)($29)
	sw		$16, (128-10*4)($29)
	lw		$8,(128+4*4)($29)
	.set	reorder

	lui		$10,0x6170		# compose sigma
	lui		$11,0x3320
	lui		$12,0x7962
	lui		$13,0x6b20
	ori		$10,$10,0x7865
	ori		$11,$11,0x646e
	ori		$12,$12,0x2d32
	ori		$13,$13,0x6574

	lw		$14, 4*0($7)
	lw		$15, 4*1($7)
	lw		$16, 4*2($7)
	lw		$17, 4*3($7)
	lw		$18, 4*4($7)
	lw		$19, 4*5($7)
	lw		$20,4*6($7)
	lw		$21,4*7($7)

	lw		$22,4*0($8)
	lw		$23,4*1($8)
	lw		$24,4*2($8)
	lw		$25,4*3($8)

	sw		$10, 4*0($29)
	sw		$11, 4*1($29)
	sw		$12, 4*2($29)
	sw		$13, 4*3($29)
	sw		$14, 4*4($29)
	sw		$15, 4*5($29)
	sw		$16, 4*6($29)
	sw		$17, 4*7($29)
	sw		$18, 4*8($29)
	sw		$19, 4*9($29)
	sw		$20,4*10($29)
	sw		$21,4*11($29)
	move		$30,$22
	sw		$23,4*13($29)
	sw		$24,4*14($29)
	sw		$25,4*15($29)

	li		$1,9
	bal		.Lalt_entry

	# A null input requests raw keystream: no input loads or XOR.
	beqz		$5,.Lkeystream

	sltiu		$1,$6,64
	or		$31,$5,$4
	andi		$31,$31,3		# both are aligned?
	bnez		$1,.Ltail

#ifndef	MIPSEB
	beqz		$31,.Loop_aligned
#endif
.Loop_misaligned:
	# On little-endian pre-R6 processor it's possible to reduce
	# amount of instructions by using lwl+lwr to load input, and
	# single 'xor' per word. Judging from sheer instruction count
	# it could give ~15% improvement. But in real life it turned
	# to be just ~5%, too little to care about...

	lbu		$2,0($5)
	lbu		$3,1($5)
	srl		$9,$10,8
	lbu		$7,2($5)
	srl		$1,$10,16
	lbu		$8,3($5)
	srl		$31,$10,24
	xor		$10,$10,$2
	lbu		$2,4+0($5)
	xor		$9,$9,$3
	lbu		$3,4+1($5)
	xor		$1,$1,$7
	lbu		$7,4+2($5)
	xor		$31,$31,$8
	lbu		$8,4+3($5)
	sb		$10,0+0($4)
	sb		$9,0+1($4)
	srl		$9,$11,8
	sb		$1,0+2($4)
	srl		$1,$11,16
	sb		$31,0+3($4)
	srl		$31,$11,24
	xor		$11,$11,$2
	lbu		$2,8+0($5)
	xor		$9,$9,$3
	lbu		$3,8+1($5)
	xor		$1,$1,$7
	lbu		$7,8+2($5)
	xor		$31,$31,$8
	lbu		$8,8+3($5)
	sb		$11,4+0($4)
	sb		$9,4+1($4)
	srl		$9,$12,8
	sb		$1,4+2($4)
	srl		$1,$12,16
	sb		$31,4+3($4)
	srl		$31,$12,24
	xor		$12,$12,$2
	lbu		$2,12+0($5)
	xor		$9,$9,$3
	lbu		$3,12+1($5)
	xor		$1,$1,$7
	lbu		$7,12+2($5)
	xor		$31,$31,$8
	lbu		$8,12+3($5)
	sb		$12,8+0($4)
	sb		$9,8+1($4)
	srl		$9,$13,8
	sb		$1,8+2($4)
	srl		$1,$13,16
	sb		$31,8+3($4)
	srl		$31,$13,24
	xor		$13,$13,$2
	lbu		$2,16+0($5)
	xor		$9,$9,$3
	lbu		$3,16+1($5)
	xor		$1,$1,$7
	lbu		$7,16+2($5)
	xor		$31,$31,$8
	lbu		$8,16+3($5)
	sb		$13,12+0($4)
	sb		$9,12+1($4)
	srl		$9,$14,8
	sb		$1,12+2($4)
	srl		$1,$14,16
	sb		$31,12+3($4)
	srl		$31,$14,24
	xor		$14,$14,$2
	lbu		$2,20+0($5)
	xor		$9,$9,$3
	lbu		$3,20+1($5)
	xor		$1,$1,$7
	lbu		$7,20+2($5)
	xor		$31,$31,$8
	lbu		$8,20+3($5)
	sb		$14,16+0($4)
	sb		$9,16+1($4)
	srl		$9,$15,8
	sb		$1,16+2($4)
	srl		$1,$15,16
	sb		$31,16+3($4)
	srl		$31,$15,24
	xor		$15,$15,$2
	lbu		$2,24+0($5)
	xor		$9,$9,$3
	lbu		$3,24+1($5)
	xor		$1,$1,$7
	lbu		$7,24+2($5)
	xor		$31,$31,$8
	lbu		$8,24+3($5)
	sb		$15,20+0($4)
	sb		$9,20+1($4)
	srl		$9,$16,8
	sb		$1,20+2($4)
	srl		$1,$16,16
	sb		$31,20+3($4)
	srl		$31,$16,24
	xor		$16,$16,$2
	lbu		$2,28+0($5)
	xor		$9,$9,$3
	lbu		$3,28+1($5)
	xor		$1,$1,$7
	lbu		$7,28+2($5)
	xor		$31,$31,$8
	lbu		$8,28+3($5)
	sb		$16,24+0($4)
	sb		$9,24+1($4)
	srl		$9,$17,8
	sb		$1,24+2($4)
	srl		$1,$17,16
	sb		$31,24+3($4)
	srl		$31,$17,24
	xor		$17,$17,$2
	lbu		$2,32+0($5)
	xor		$9,$9,$3
	lbu		$3,32+1($5)
	xor		$1,$1,$7
	lbu		$7,32+2($5)
	xor		$31,$31,$8
	lbu		$8,32+3($5)
	sb		$17,28+0($4)
	sb		$9,28+1($4)
	srl		$9,$18,8
	sb		$1,28+2($4)
	srl		$1,$18,16
	sb		$31,28+3($4)
	srl		$31,$18,24
	xor		$18,$18,$2
	lbu		$2,36+0($5)
	xor		$9,$9,$3
	lbu		$3,36+1($5)
	xor		$1,$1,$7
	lbu		$7,36+2($5)
	xor		$31,$31,$8
	lbu		$8,36+3($5)
	sb		$18,32+0($4)
	sb		$9,32+1($4)
	srl		$9,$19,8
	sb		$1,32+2($4)
	srl		$1,$19,16
	sb		$31,32+3($4)
	srl		$31,$19,24
	xor		$19,$19,$2
	lbu		$2,40+0($5)
	xor		$9,$9,$3
	lbu		$3,40+1($5)
	xor		$1,$1,$7
	lbu		$7,40+2($5)
	xor		$31,$31,$8
	lbu		$8,40+3($5)
	sb		$19,36+0($4)
	sb		$9,36+1($4)
	srl		$9,$20,8
	sb		$1,36+2($4)
	srl		$1,$20,16
	sb		$31,36+3($4)
	srl		$31,$20,24
	xor		$20,$20,$2
	lbu		$2,44+0($5)
	xor		$9,$9,$3
	lbu		$3,44+1($5)
	xor		$1,$1,$7
	lbu		$7,44+2($5)
	xor		$31,$31,$8
	lbu		$8,44+3($5)
	sb		$20,40+0($4)
	sb		$9,40+1($4)
	srl		$9,$21,8
	sb		$1,40+2($4)
	srl		$1,$21,16
	sb		$31,40+3($4)
	srl		$31,$21,24
	xor		$21,$21,$2
	lbu		$2,48+0($5)
	xor		$9,$9,$3
	lbu		$3,48+1($5)
	xor		$1,$1,$7
	lbu		$7,48+2($5)
	xor		$31,$31,$8
	lbu		$8,48+3($5)
	sb		$21,44+0($4)
	sb		$9,44+1($4)
	srl		$9,$22,8
	sb		$1,44+2($4)
	srl		$1,$22,16
	sb		$31,44+3($4)
	srl		$31,$22,24
	xor		$22,$22,$2
	lbu		$2,52+0($5)
	xor		$9,$9,$3
	lbu		$3,52+1($5)
	xor		$1,$1,$7
	lbu		$7,52+2($5)
	xor		$31,$31,$8
	lbu		$8,52+3($5)
	sb		$22,48+0($4)
	sb		$9,48+1($4)
	srl		$9,$23,8
	sb		$1,48+2($4)
	srl		$1,$23,16
	sb		$31,48+3($4)
	srl		$31,$23,24
	xor		$23,$23,$2
	lbu		$2,56+0($5)
	xor		$9,$9,$3
	lbu		$3,56+1($5)
	xor		$1,$1,$7
	lbu		$7,56+2($5)
	xor		$31,$31,$8
	lbu		$8,56+3($5)
	sb		$23,52+0($4)
	sb		$9,52+1($4)
	srl		$9,$24,8
	sb		$1,52+2($4)
	srl		$1,$24,16
	sb		$31,52+3($4)
	srl		$31,$24,24
	xor		$24,$24,$2
	lbu		$2,60+0($5)
	xor		$9,$9,$3
	lbu		$3,60+1($5)
	xor		$1,$1,$7
	lbu		$7,60+2($5)
	xor		$31,$31,$8
	lbu		$8,60+3($5)
	sb		$24,56+0($4)
	sb		$9,56+1($4)
	srl		$9,$25,8
	sb		$1,56+2($4)
	srl		$1,$25,16
	sb		$31,56+3($4)
	srl		$31,$25,24
	xor		$25,$25,$2
	xor		$9,$9,$3
	xor		$1,$1,$7
	xor		$31,$31,$8
	sb		$25,60($4)
	addu		$30,$30,1		# next counter value
	sb		$9,61($4)
	subu	$6,$6,64
	sb		$1,62($4)
	addu	$5,$5,64
	sb		$31,63($4)
	addu	$4,$4,64
	beqz		$6,.Ldone

	sltiu		$9,$6,64
	li		$1,9
	bal		__ChaCha

	beqz		$9,.Loop_misaligned

#ifndef	MIPSEB
	b		.Ltail

.align	4
.Loop_aligned:
	lw		$2,0($5)
	lw		$3,4($5)
	lw		$7,8($5)
	lw		$8,12($5)
	xor		$10,$10,$2
	lw		$2,16+0($5)
	xor		$11,$11,$3
	lw		$3,16+4($5)
	xor		$12,$12,$7
	lw		$7,16+8($5)
	xor		$13,$13,$8
	lw		$8,16+12($5)
	sw		$10,0+0($4)
	sw		$11,0+4($4)
	sw		$12,0+8($4)
	sw		$13,0+12($4)
	xor		$14,$14,$2
	lw		$2,32+0($5)
	xor		$15,$15,$3
	lw		$3,32+4($5)
	xor		$16,$16,$7
	lw		$7,32+8($5)
	xor		$17,$17,$8
	lw		$8,32+12($5)
	sw		$14,16+0($4)
	sw		$15,16+4($4)
	sw		$16,16+8($4)
	sw		$17,16+12($4)
	xor		$18,$18,$2
	lw		$2,48+0($5)
	xor		$19,$19,$3
	lw		$3,48+4($5)
	xor		$20,$20,$7
	lw		$7,48+8($5)
	xor		$21,$21,$8
	lw		$8,48+12($5)
	sw		$18,32+0($4)
	sw		$19,32+4($4)
	sw		$20,32+8($4)
	sw		$21,32+12($4)
	xor		$22,$22,$2
	xor		$23,$23,$3
	xor		$24,$24,$7
	xor		$25,$25,$8
	sw		$22,48($4)
	addu		$30,$30,1		# next counter value
	sw		$23,52($4)
	subu	$6,$6,64
	sw		$24,56($4)
	addu	$5,$5,64
	sw		$25,60($4)
	addu	$4,$4,64
	sltiu		$9,$6,64
	beqz		$6,.Ldone

	li		$1,9
	bal		__ChaCha

	beqz		$9,.Loop_aligned
#endif

	b		.Ltail

.align 4
.Lkeystream:
	sltiu		$1,$6,64
	bnez		$1,.Ltail
#ifndef MIPSEB
	andi		$1,$4,3
	bnez		$1,.Lkeystream_unaligned
	sw		$10,0($4)
	sw		$11,4($4)
	sw		$12,8($4)
	sw		$13,12($4)
	sw		$14,16($4)
	sw		$15,20($4)
	sw		$16,24($4)
	sw		$17,28($4)
	sw		$18,32($4)
	sw		$19,36($4)
	sw		$20,40($4)
	sw		$21,44($4)
	sw		$22,48($4)
	sw		$23,52($4)
	sw		$24,56($4)
	sw		$25,60($4)
	b		.Lkeystream_next
#endif
.Lkeystream_unaligned:
	sb		$10,0($4)
	srl		$10,$10,8
	sb		$10,1($4)
	srl		$10,$10,8
	sb		$10,2($4)
	srl		$10,$10,8
	sb		$10,3($4)
	sb		$11,4($4)
	srl		$11,$11,8
	sb		$11,5($4)
	srl		$11,$11,8
	sb		$11,6($4)
	srl		$11,$11,8
	sb		$11,7($4)
	sb		$12,8($4)
	srl		$12,$12,8
	sb		$12,9($4)
	srl		$12,$12,8
	sb		$12,10($4)
	srl		$12,$12,8
	sb		$12,11($4)
	sb		$13,12($4)
	srl		$13,$13,8
	sb		$13,13($4)
	srl		$13,$13,8
	sb		$13,14($4)
	srl		$13,$13,8
	sb		$13,15($4)
	sb		$14,16($4)
	srl		$14,$14,8
	sb		$14,17($4)
	srl		$14,$14,8
	sb		$14,18($4)
	srl		$14,$14,8
	sb		$14,19($4)
	sb		$15,20($4)
	srl		$15,$15,8
	sb		$15,21($4)
	srl		$15,$15,8
	sb		$15,22($4)
	srl		$15,$15,8
	sb		$15,23($4)
	sb		$16,24($4)
	srl		$16,$16,8
	sb		$16,25($4)
	srl		$16,$16,8
	sb		$16,26($4)
	srl		$16,$16,8
	sb		$16,27($4)
	sb		$17,28($4)
	srl		$17,$17,8
	sb		$17,29($4)
	srl		$17,$17,8
	sb		$17,30($4)
	srl		$17,$17,8
	sb		$17,31($4)
	sb		$18,32($4)
	srl		$18,$18,8
	sb		$18,33($4)
	srl		$18,$18,8
	sb		$18,34($4)
	srl		$18,$18,8
	sb		$18,35($4)
	sb		$19,36($4)
	srl		$19,$19,8
	sb		$19,37($4)
	srl		$19,$19,8
	sb		$19,38($4)
	srl		$19,$19,8
	sb		$19,39($4)
	sb		$20,40($4)
	srl		$20,$20,8
	sb		$20,41($4)
	srl		$20,$20,8
	sb		$20,42($4)
	srl		$20,$20,8
	sb		$20,43($4)
	sb		$21,44($4)
	srl		$21,$21,8
	sb		$21,45($4)
	srl		$21,$21,8
	sb		$21,46($4)
	srl		$21,$21,8
	sb		$21,47($4)
	sb		$22,48($4)
	srl		$22,$22,8
	sb		$22,49($4)
	srl		$22,$22,8
	sb		$22,50($4)
	srl		$22,$22,8
	sb		$22,51($4)
	sb		$23,52($4)
	srl		$23,$23,8
	sb		$23,53($4)
	srl		$23,$23,8
	sb		$23,54($4)
	srl		$23,$23,8
	sb		$23,55($4)
	sb		$24,56($4)
	srl		$24,$24,8
	sb		$24,57($4)
	srl		$24,$24,8
	sb		$24,58($4)
	srl		$24,$24,8
	sb		$24,59($4)
	sb		$25,60($4)
	srl		$25,$25,8
	sb		$25,61($4)
	srl		$25,$25,8
	sb		$25,62($4)
	srl		$25,$25,8
	sb		$25,63($4)
.Lkeystream_next:
	addu		$30,$30,1
	subu		$6,$6,64
	addu		$4,$4,64
	beqz		$6,.Ldone
	li		$1,9
	bal		__ChaCha
	b		.Lkeystream

.Ltail:
	move		$30,$29
	sw		$11, 4*1($29)
	sw		$12, 4*2($29)
	sw		$13, 4*3($29)
	sw		$14, 4*4($29)
	sw		$15, 4*5($29)
	sw		$16, 4*6($29)
	sw		$17, 4*7($29)
	sw		$18, 4*8($29)
	sw		$19, 4*9($29)
	sw		$20,4*10($29)
	sw		$21,4*11($29)
	sw		$22,4*12($29)
	sw		$23,4*13($29)
	sw		$24,4*14($29)
	sw		$25,4*15($29)


	beqz		$5,.Lkeystream_tail
	b		.Loop_tail
.Lkeystream_tail:
	beqz		$6,.Ldone
	li		$9,4
.Lkeystream_tail_word:
	sb		$10,0($4)
	srl		$10,$10,8
	addu		$4,$4,1
	subu		$6,$6,1
	beqz		$6,.Ldone
	subu		$9,$9,1
	bnez		$9,.Lkeystream_tail_word
	addu		$30,$30,4
	lw		$10,0($30)
	b		.Lkeystream_tail

.Loop_tail:
	sltiu		$1,$6,4
	addu	$30,$30,4
	bnez		$1,.Last_word

	lbu		$2,0($5)
	lbu		$3,1($5)
	lbu		$7,2($5)
	subu	$6,$6,4
	lbu		$8,3($5)
	addu	$5,$5,4
	xor		$2,$2,$10
	srl		$10,$10,8
	xor		$3,$3,$10
	srl		$10,$10,8
	xor		$7,$7,$10
	srl		$10,$10,8
	xor		$8,$8,$10
	lw		$10,0($30)
	sb		$2,0($4)
	sb		$3,1($4)
	sb		$7,2($4)
	sb		$8,3($4)
	addu	$4,$4,4
	b		.Loop_tail

	.set	noreorder
.Last_word:
	beqz		$6,.Ldone
	subu	$6,$6,1
	lbu		$2,0($5)
	addu	$5,$5,1
	xor		$2,$2,$10
	srl		$10,$10,8
	sb		$2,0($4)
	b		.Last_word
	addu	$4,$4,1

.align	4
.Ldone:
	lw		$31, (128-1*4)($29)
	lw		$30, (128-2*4)($29)
	lw		$23,(128-3*4)($29)
	lw		$22,(128-4*4)($29)
	lw		$21, (128-5*4)($29)
	lw		$20, (128-6*4)($29)
	lw		$19, (128-7*4)($29)
	lw		$18, (128-8*4)($29)
	lw		$17, (128-9*4)($29)
	lw		$16, (128-10*4)($29)
	jr		$31
	addu	$29,$29,128
.end	ChaCha20_ctr32
