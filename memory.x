MEMORY {
    DDR : ORIGIN = 0x10000000, LENGTH = 256M
}

REGION_ALIAS("VECTORS", DDR);
REGION_ALIAS("CODE", DDR);
REGION_ALIAS("DATA", DDR);
REGION_ALIAS("STACKS", DDR);

PROVIDE(_hyp_stack_size = 0);
PROVIDE(_und_stack_size = 16K);
PROVIDE(_svc_stack_size = 16K);
PROVIDE(_abt_stack_size = 16K);
PROVIDE(_irq_stack_size = 16K);
PROVIDE(_fiq_stack_size = 0);
PROVIDE(_sys_stack_size = 16K);

PROVIDE(_num_cores = 2);
