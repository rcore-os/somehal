#![allow(unused)]

#[repr(usize)]
pub enum DcacheOp {
    CleanAndInvalidate = 0,
    InvalidateOnly = 1,
}

use core::arch::naked_asm;

#[unsafe(naked)]
pub unsafe extern "C" fn flush_dcache_range(start: usize, end: usize) {
    naked_asm!(
        "
    mrs	x3, ctr_el0
	ubfx	x3, x3, #16, #4
	mov	x2, #4
	lsl	x2, x2, x3		/* cache line size */

	/* x2 <- minimal cache line size in cache system */
	sub	x3, x2, #1
	bic	x0, x0, x3
1:	dc	civac, x0	/* clean & invalidate data or unified cache */
	add	x0, x0, x2
	cmp	x0, x1
	b.lo	1b
	dsb	sy
	ret
            "
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn flush_invalidate_range(start: usize, end: usize) {
    naked_asm!(
        "
    mrs	    x3, ctr_el0
	ubfx	x3, x3, #16, #4
	mov	    x2, #4
	lsl	    x2, x2, x3		/* cache line size */

	/* x2 <- minimal cache line size in cache system */
	sub	x3, x2, #1
	bic	x0, x0, x3
1:	dc	ivac, x0	/* invalidate data or unified cache */
	add	x0, x0, x2
	cmp	x0, x1
	b.lo	1b
	dsb	sy
	ret
            "
    )
}

#[unsafe(naked)]
pub unsafe extern "C" fn invalidate_icache_all() {
    naked_asm!(
        "
    ic	ialluis
	isb	sy
	ret
            "
    )
}

/// Flush and invalidate all cache levels
///
/// x16: FEAT_CCIDX
/// x2~x9: clobbered
#[unsafe(naked)]
pub unsafe extern "C" fn dcache_level(cache_level: usize, op: DcacheOp) {
    naked_asm!(
        "
	lsl	x12, x0, #1
	msr	csselr_el1, x12		/* select cache level */
	isb				/* sync change of cssidr_el1 */
	mrs	x6, ccsidr_el1		/* read the new cssidr_el1 */
	ubfx	x2, x6,  #0,  #3	/* x2 <- log2(cache line size)-4 */
	cbz	x16, 3f			/* check for FEAT_CCIDX */
	ubfx	x3, x6,  #3, #21	/* x3 <- number of cache ways - 1 */
	ubfx	x4, x6, #32, #24	/* x4 <- number of cache sets - 1 */
	b	4f
3:
	ubfx	x3, x6,  #3, #10	/* x3 <- number of cache ways - 1 */
	ubfx	x4, x6, #13, #15	/* x4 <- number of cache sets - 1 */
4:
	add	x2, x2, #4		/* x2 <- log2(cache line size) */
	clz	w5, w3			/* bit position of #ways */
	/* x12 <- cache level << 1 */
	/* x2 <- line length offset */
	/* x3 <- number of cache ways - 1 */
	/* x4 <- number of cache sets - 1 */
	/* x5 <- bit position of #ways */

5:
	mov	x6, x3			/* x6 <- working copy of #ways */
6:
	lsl	x7, x6, x5
	orr	x9, x12, x7		/* map way and level to cisw value */
	lsl	x7, x4, x2
	orr	x9, x9, x7		/* map set number to cisw value */
	tbz	w1, #0, 1f
	dc	isw, x9
	b	2f
1:	dc	cisw, x9		/* clean & invalidate by set/way */
2:	subs	x6, x6, #1		/* decrement the way */
	b.ge	6b
	subs	x4, x4, #1		/* decrement the set */
	b.ge	5b

	ret
            "
    )
}

/// Flush or invalidate all data cache by SET/WAY.
#[unsafe(naked)]
pub unsafe extern "C" fn dcache_all(op: DcacheOp) {
    naked_asm!(
                "
	mov	x1, x0
	dsb	sy
	mrs	x10, clidr_el1		/* read clidr_el1 */
	ubfx	x11, x10, #24, #3	/* x11 <- loc */
	cbz	x11, 3b		/* if loc is 0, exit */
	mov	x15, lr
	mrs	x16, s3_0_c0_c7_2	/* read value of id_aa64mmfr2_el1*/
	ubfx	x16, x16, #20, #4	/* save FEAT_CCIDX identifier in x16 */
	mov	x0, #0			/* start flush at cache level 0 */
	/* x0  <- cache level */
	/* x10 <- clidr_el1 */
	/* x11 <- loc */
	/* x15 <- return address */
/* loop level */
1:
	add	x12, x0, x0, lsl #1	/* x12 <- tripled cache level */
	lsr	x12, x10, x12
	and	x12, x12, #7		/* x12 <- cache type */
	cmp	x12, #2
	b.lt	2b			/* skip if no cache or icache */
	bl	{dcache_level}	/* x1 = 0 flush, 1 invalidate */
/* skip */
2:
	add	x0, x0, #1		/* increment cache level */
	cmp	x11, x0
	b.gt	1b

	mov	x0, #0
	msr	csselr_el1, x0		/* restore csselr_el1 */
	dsb	sy
	isb
	mov	lr, x15
/* finished */
3:
	ret
            ",
    dcache_level = sym dcache_level
            )
}
