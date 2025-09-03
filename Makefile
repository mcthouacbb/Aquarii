ifndef EXE
	EXE = aquarii
endif

EXE_SUFFIX =

ifeq ($(OS), WINDOWS_NT)
	EXE_SUFFIX = .exe
endif

default:
	cargo rustc --release --bin aquarii -- -C target-cpu=native --emit link=$(EXE)$(EXE_SUFFIX)

datagen:
	cargo rustc --release --features datagen --bin aquarii -- -C target-cpu=native --emit link=$(EXE)-datagen$(EXE_SUFFIX)
