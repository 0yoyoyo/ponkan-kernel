TARGET = libdriver.a
OBJS = lib.o interface.o newlib_support.o libcxx_support.o logger.o \
       usb/memory.o usb/device.o usb/xhci/ring.o usb/xhci/trb.o usb/xhci/xhci.o \
       usb/xhci/port.o usb/xhci/device.o usb/xhci/devmgr.o usb/xhci/registers.o \
       usb/classdriver/base.o usb/classdriver/hid.o usb/classdriver/keyboard.o \
       usb/classdriver/mouse.o
DEPENDS = $(join $(dir $(OBJS)),$(addprefix .,$(notdir $(OBJS:.o=.d))))

BASEDIR = $(HOME)/osbook/mikanos-build/devenv/x86_64-elf

CPPFLAGS  = -I$(BASEDIR)/include/c++/v1 -I$(BASEDIR)/include -nostdlibinc \
            -nostdlibinc -D__ELF__ -D_LDBL_EQ_DBL -D_GNU_SOURCE -D_POSIX_TIMERS
CPPFLAGS += -I.
CFLAGS   += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone
CXXFLAGS += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone \
            -fno-exceptions -fno-rtti -std=c++17

.PHONY: all
all: $(TARGET)

.PHONY: clean
clean:
	find . -name "*.o" | xargs rm -f
	find . -name ".*.d" | xargs rm -f
	rm -f *.a

$(TARGET): libusb.a Makefile
	ln -sf $(BASEDIR)/lib/libc++.a libcxx.a
	echo \
		"create $(TARGET)\n" \
		"addlib libusb.a\n" \
		"addlib $(BASEDIR)/lib/libc.a\n" \
		"addlib libcxx.a\n" \
		"save\n" \
		| ar -M

libusb.a: $(OBJS) Makefile
	ar rcs libusb.a $(OBJS)

%.o: %.cpp Makefile
	clang++ $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

.%.d: %.cpp
	clang++ $(CPPFLAGS) $(CXXFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.cpp=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

%.o: %.c Makefile
	clang $(CPPFLAGS) $(CFLAGS) -c $< -o $@

.%.d: %.c
	clang $(CPPFLAGS) $(CFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.c=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

.PHONY: depends
depends:
	$(MAKE) $(DEPENDS)

-include $(DEPENDS)
