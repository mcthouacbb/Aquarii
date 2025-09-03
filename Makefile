ifndef EXE
	EXE = aquarii
endif

ifeq ($(OS), Windows_NT)
	EXE_SUFFIX := .exe
endif

default:
	cargo rustc --release --bin aquarii -- -C target-cpu=native --emit link=$(EXE)$(EXE_SUFFIX)

datagen:
	$(info $(EXE_SUFFIX))
	cargo rustc --release --features datagen --bin aquarii -- -C target-cpu=native --emit link=$(EXE)-datagen$(EXE_SUFFIX)
