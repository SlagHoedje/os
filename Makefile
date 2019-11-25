arch ?= x86_64
target ?= $(arch)-os
kernel := target/kernel-$(arch).bin
iso := os-$(arch).iso

linker_script := src/boot/linker.ld
grub_cfg := src/boot/grub.cfg
asm_src := $(wildcard src/boot/*.asm)
asm_obj := $(patsubst src/boot/%.asm, target/boot/%.o, $(asm_src))
rust_obj := target/$(target)/debug/libos.a

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@cargo.exe clean
	@rm -f $(iso)

run: $(iso)
	@echo " -- QEMU / SERIAL OUTPUT --"
	@qemu-system-x86_64.exe -sdl -cdrom $(iso) -s -serial stdio

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@echo "Creating iso file..."
	@mkdir -p target/isofiles/boot/grub
	@cp $(kernel) target/isofiles/boot/kernel.bin
	@cp $(grub_cfg) target/isofiles/boot/grub
	@grub-mkrescue -o $(iso) target/isofiles 2> /dev/null
	@rm -rf build/isofiles

$(kernel): kernel $(rust_obj) $(asm_obj) $(linker_script)
	@echo "Compiling boot code..."
	@ld -n --gc-sections --strip-debug -T $(linker_script) -o $(kernel) $(asm_obj) $(rust_obj)

kernel:
	@echo "Compiling rust code..."
	@cmd.exe /V /C "set RUST_TARGET_PATH=E:/Programming/Rust/os&& cargo.exe xbuild --target $(target)"

target/boot/%.o: src/boot/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@