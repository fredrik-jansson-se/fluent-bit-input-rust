INCLUDES += -I fluent-bit/include
INCLUDES += -I fluent-bit/lib/monkey/include
INCLUDES += -I fluent-bit/lib/msgpack-c/include
INCLUDES += -I fluent-bit/lib/flb_libco
INCLUDES += -I fluent-bit/lib/mbedtls-2.27.0/include
INCLUDES += -I fluent-bit/lib/c-ares-809d5e84/include
INCLUDES += -I fluent-bit/build/lib/c-ares-809d5e84
INCLUDES += -I fluent-bit/lib/cmetrics/include
INCLUDES += -I fluent-bit/lib/cmetrics/lib/mpack/src
INCLUDES += -I fluent-bit/lib/

CFLAGS += -m64 -O2
LDFLAGS += -shared

FLUENT_BIT = fluent-bit/build/bin/fluent-bit

fluent-bit:
	mkdir -p $@
	curl -L https://github.com/fluent/fluent-bit/archive/refs/tags/v1.8.11.tar.gz | tar xfz - -C $@ --strip-components=1

$(FLUENT_BIT): fluent-bit
	cd fluent-bit/build && cmake .. -DCMAKE_BUILD_TYPE=RELEASE && make all

RSOURCES = $(wildcard src/*rs)
SOURCES = $(wildcard csrc/*c)
OBJECTS = $(patsubst csrc/%,out/%,$(SOURCES:.c=.o))
TARGET = flb-in_example.so

out/%.o: csrc/%.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -fpic -c $< -o $@

$(TARGET): $(FLUENT_BIT) $(OBJECTS) $(RSOURCES)
	cargo build --release
	$(CC) -s $(CFLAGS) $(LDFLAGS) -o $@ $(OBJECTS) target/release/libin_example.a

.PHONY: all clean real-clean run
all: $(TARGET)

clean:
	rm -rf out $(TARGET)

real-clean: clean
	cargo clean
	rm -rf fluent-bit

run: 
	fluent-bit/build/bin/fluent-bit -vv -e ./flb-in_example.so -i example -o stdout
