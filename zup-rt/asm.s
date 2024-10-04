  .section .start, "ax"
  .type start, %function
  .global start
start:
  /* initialize registers */
  mov r0,#0
  mov r1,#0
  mov r2,#0
  mov r3,#0
  mov r4,#0
  mov r5,#0
  mov r6,#0
  mov r7,#0
  mov r8,#0
  mov r9,#0
  mov r10,#0
  mov r11,#0
  mov r12,#0
  ldr sp,=__stack_top__        /* initialize the stack pointer */
  mov lr,#0
  mrc p15, 0, r0, c1, c0, 1    /* read ACTLR */
  /*  FIXME properly initialize ECC instead of disabling it */
  bic r0, r0, #7 << 25         /* disable ATCM, BTCM{0,1} ECC */
  mcr p15, 0, r0, cr1, cr0, 1  /* write ACTLR */

  /* region 0: all memory 0x0 -- 0x1_0000_0000 */
  mov r0,#0                    // 0
  mcr p15, 0, r0, c6, c2, 0    // region number
  mov r0,#0                    // 0x0000_0000
  mcr p15, 0, r0, c6, c1, 0    // base address
  mov r0,#0x3f                 // 4 GiB, all memory
  mcr p15, 0, r0, c6, c1, 2    // size & enable
  mov r0,#0x30b                // normal write-back write-allocate non-shareable full-access
  mcr p15, 0, r0, c6, c1, 4    // access control

  /* region 1: presto-core 0x8000_0000 -- 0xa000_0000 */
  mov r0,#1                    // 1
  mcr p15, 0, r0, c6, c2, 0    // region number
  mov r0,#0x80000000           // 0x8000_0000
  mcr p15, 0, r0, c6, c1, 0    // base address
  mov r0,#0x39                 // 512 MiB
  mcr p15, 0, r0, c6, c1, 2    // size & enable
  mov r0,#0x1305               // device shareable full-access execute-never
  mcr p15, 0, r0, c6, c1, 4    // access control

  /* region 2: OCM 0xfffc_0000 -- 0x1_0000_0000 */
  mov r0,#2                    // 2
  mcr p15, 0, r0, c6, c2, 0    // region number
  //mov r0,#0xfffc0000         // 0xfffc_0000 has too many bits!
  mov r0,#0xff000000           // first top 8 bits
  orr r0,r0,#0x00fc0000        // then OR-in the rest
  mcr p15, 0, r0, c6, c1, 0    // base address
  mov r0,#0x23                 // 256 kiB
  mcr p15, 0, r0, c6, c1, 2    // size & enable
  //mov r0,#0x302                // normal write-through read-allocate non-shareable full access
  mov r0,#0x1305               // device shareable full-access execute-never
  mcr p15, 0, r0, c6, c1, 4    // access control

  /* region 3: ATCM1 0x0000_8000 -- 0x0001_0000 */
  mov r0,#3                    // 3
  mcr p15, 0, r0, c6, c2, 0    // region number
  mov r0,#0x00008000           // 0x0000_8000
  mcr p15, 0, r0, c6, c1, 0    // base address
  mov r0,#0x1d                 // 32 kiB
  mcr p15, 0, r0, c6, c1, 2    // size & enable
  mov r0,#0x1305               // device shareable full-access execute-never
  mcr p15, 0, r0, c6, c1, 4    // access control

  dsb                          /* data synchronization barrier */
  mrc p15, 0, r0, c1, c0, 0    /* read SCTLR */
  bic r0, r0, #1 << 13         /* clear V bit to map the vector table to address 0 */
  orr r0, r0, #1 << 0          /* MPU enable */
  orr r0, r0, #1 << 12         /* instruction cache enable */
  orr r0, r0, #1 << 2          /* data cache enable */
  dsb                          /* data synchronization barrier */
  mcr p15, 0, r1, c15, c5, 0   /* invalidate entire data cache */
  mcr p15, 0, r1, c7, c5, 0    /* invalidate entire instruction cache */
  mcr p15, 0, r0, cr1, cr0, 0  /* write SCTLR */
  isb                          /* instruction synchronization barrier */

  /* enable VFP support (FPU) */
  mrc p15, 0, r0, c1, c0, 2    // Read CPACR
  orr r0, r0, #15 << 20        // cp10 and cp11 Privileged and User mode access (bits 20-23)
  mcr p15, 0, r0, c1, c0, 2    // Write CPACR
  isb                          /* instruction synchronization barrier */
  mov r0, #1 << 30             // set EN bit in FPEXC register (bit 30)
  vmsr fpexc, r0               // Write Floating-point Exception Control Register

  /* user enable register */
  mov r0, #1                   // set EN bit in PMUSERENR register (bit 0)
  MCR p15, 0, r0, c9, c14, 0   // Write PMUSERENR Register

  /* enable cycle counter */
  MRC p15, 0, r0, c9, c12, 1   // Read PMCNTENSET Register
  orr r0, r0, #1 << 31         // Set bit 31: C: Cycle counter enable
  MCR p15, 0, r0, c9, c12, 1   // Write PMCNTENSET Register

  MRC p15, 0, r0, c9, c12, 0   // Read PMCR Register
  orr r0, r0, #1               // Set bit 1: enable all counter
  MCR p15, 0, r0, c9, c12, 0   // Write PMCR Register

  b main

  .section .vectors, "ax"
  .type Vectors, %function
  .global Vectors
Vectors:
  ldr pc,=start                     /* 0x00 */
  ldr pc,=UndefinedTrampoline       /* 0x04 */
  ldr pc,=SVCTrampoline             /* 0x08 */
  ldr pc,=PrefetchAbortTrampoline   /* 0x0C */
  ldr pc,=DataAbortTrampoline       /* 0x10 */
  nop                               /* 0x14 */
  ldr pc,=IRQTrampoline             /* 0x18 */
  ldr pc,=FIQTrampoline             /* 0x1C */

  .section .text.UndefinedTrampoline, "ax"
  .type UndefinedTrampoline, %function
  .global UndefinedTrampoline
UndefinedTrampoline:
  cps #19 /* switch back to the supervisor mode to reuse the previous stack */
  b Undefined

  .section .text.SVCTrampoline, "ax"
  .type SVCTrampoline, %function
  .global SVCTrampoline
SVCTrampoline:
  cps #19 /* switch back to the supervisor mode to reuse the previous stack */
  b SVC

  .section .text.PrefetchAbortTrampoline, "ax"
  .type PreftechAbortTrampoline, %function
  .global PrefetchAbortTrampoline
PrefetchAbortTrampoline:
  cps #19 /* switch back to the supervisor mode to reuse the previous stack */
  b PrefetchAbort

  .section .text.DataAbortTrampoline, "ax"
  .type DataAbortTrampoline, %function
  .global DataAbortTrampoline
DataAbortTrampoline:
  cps #19 /* switch back to the supervisor mode to reuse the previous stack */
  b DataAbort

/* Reentrant IRQ handler */
/* Reference: Section 6.12 Reentrant interrupt handlers of "ARM Compiler
   Toolchain Developing Software for ARM Processors" */
  .section .text.IRQTrampoline, "ax"
  .type IRQTrampoline, %function
  .global IRQTrampoline
IRQTrampoline:
  sub lr, lr, #4        /* construct the return address */
  srsdb sp!, #19        /* save LR_irq and SPSR_irq to Supervisor mode stack */
  cps #19               /* switch to Supervisor mode */
  push {r0-r3, ip}      /* push other AAPCS registers */
  and r1, sp, #4        /* test alignment of the stack */
  sub sp, sp, r1        /* remove any misalignment (0 or 4) */
  push {r1, lr}         /* push the adjustment and lr_USR */
  movw r0, #4108
  movt r0, #63744
  ldr r0, [r0]          /* read ICCIAR */
  push {r0}
  cpsie i               /* enable IRQ */
  bl IRQ                /* call IRQ(<ICCIAR>) */
  cpsid i               /* disable IRQ */
  /* ICC::set_icceoir(icciar); */
  pop {r1}
  movw r0, #4112
  movt r0, #63744
  str r1, [r0]
  pop {r1, lr}          /* pop stack adjustment and lr_USR */
  add sp, sp, r1        /* add the stack adjustment (0 or 4) */
  pop {r0-r3, ip}       /* pop registers */
  rfeia sp!             /* return using RFE from System mode stack */

  .section .text.FIQTrampoline, "ax"
  .type FIQTrampoline, %function
  .global FIQTrampoline
FIQTrampoline:
  cps #19 /* switch back to the supervisor mode to reuse the previous stack */
  b FIQ
