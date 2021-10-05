TARGET = libhankaku.a

.PHONY: all
all: $(TARGET)

.PHONY: clean
clean:
	rm -f hankaku.bin hankaku.o libhankaku.a

$(TARGET): hankaku.txt Makefile
	./makefont.py -o hankaku.bin hankaku.txt
	objcopy -I binary -O elf64-x86-64 -B i386:x86-64 hankaku.bin hankaku.o
	ar rcs $(TARGET) hankaku.o
