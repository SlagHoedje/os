arch ?= x86_64
target ?= $(arch)-os
kernel := target/kernel-$(arch).bin
iso := os-$(arch).iso

linker_script := src/boot/linker.ld
grub_cfg := src/boot/grub.cfg
asm_src := $(wildcard src/boot/*.asm)
asm_obj := $(patsubst src/boot/%.asm, target/boot/%.o, $(asm_src))
rust_obj := target/$(target)/debug/libos.a

grub ?= grub
qemu ?= qemu-system-$(arch).exe
cargo ?= cargo.exe

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@cargo.exe clean
	@rm -f $(iso)

run: $(iso)
	@echo " -- QEMU / SERIAL OUTPUT --"
	@$(qemu) -sdl -cdrom $(iso) -s -serial stdio

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@echo "[making iso]"
	@mkdir -p target/isofiles/boot/grub
	@cp $(kernel) target/isofiles/boot/kernel.bin
	@cp $(grub_cfg) target/isofiles/boot/grub
	@$(grub)-mkrescue -o $(iso) target/isofiles 2> /dev/null
	@rm -rf target/isofiles

$(kernel): kernel $(rust_obj) $(asm_obj) $(linker_script)
	@echo "[linking]"
	@ld -n --gc-sections --strip-debug -T $(linker_script) -o $(kernel) $(asm_obj) $(rust_obj)

kernel:
	@echo "[cargo]"
	@cmd.exe /V /C "set RUST_TARGET_PATH=E:/Programming/Rust/os&& $(cargo) xbuild --target $(target)"

target/boot/%.o: src/boot/%.asm
	@echo [nasm $<]
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@