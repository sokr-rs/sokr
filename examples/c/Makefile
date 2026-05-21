# Build SOKR C example
#
# Usage:
#   make              # Build the example
#   make run          # Build and run
#   make clean        # Clean build artifacts

EXAMPLE := hello_compute
CFLAGS := -Wall -Wextra -O2 -I../../include
LDFLAGS := -L../../target/debug
LIBS := -lsokr_cpu -lsokr

.PHONY: all run clean

all: $(EXAMPLE)

$(EXAMPLE): $(EXAMPLE).c
	gcc $(CFLAGS) $(LDFLAGS) -o $@ $< $(LIBS)

run: $(EXAMPLE)
	LD_LIBRARY_PATH=../../target/debug:$$LD_LIBRARY_PATH ./$(EXAMPLE)

clean:
	rm -f $(EXAMPLE) *.o
