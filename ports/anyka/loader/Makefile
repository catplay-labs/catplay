obj-m := catplay_anyka_kexec.o
catplay_anyka_kexec-y := driver.o arm.o core.o load.o relocate.o
ccflags-y := -DCONFIG_KEXEC=1 -DCONFIG_KEXEC_CORE=1 -I$(srctree)/kernel
