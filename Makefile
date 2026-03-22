# Boot Makefile
# See https://www.gnu.org/software/make/manual/make.html for more about make.

# ENGINE
ENGINE_BUILD = g++
ENGINE_WATCH = clang++
ENGINE_FLAGS = -std=c++2b

# PATH
PATH_TEST_BIN = $(CURDIR)/build/main
PATH_TEST_SRC_RPC = $(CURDIR)/tests/rpc/main.cpp
PATH_TEST_SRC_UMQT = $(CURDIR)/tests/transport/umqt.cpp

# INC
INC_CPP = -I $(CURDIR)/src
INC_INCLUDE = -L /usr/include
INC_JSONCPP = -I /usr/include/jsoncpp/json

INC = ${INC_CPP} ${INC_INCLUDE} ${INC_JSONCPP}

# LINK
LINK_NATIVE = -Lnative/target/release -larnelify_broker -Wl,-rpath,native/target/release
LINK_JSONCPP = -ljsoncpp
LINK = ${LINK_NATIVE} ${LINK_JSONCPP}

build:
	clear && cd native \
	&& cargo build --release \
	&& cd ..

test_rpc:
	clear && mkdir -p build && rm -rf build/*
	${ENGINE_WATCH} $(ENGINE_FLAGS) ${INC} $(PATH_TEST_SRC_RPC) ${LINK} -o $(PATH_TEST_BIN) && $(PATH_TEST_BIN)

test_umqt:
	clear && mkdir -p build && rm -rf build/*
	${ENGINE_WATCH} $(ENGINE_FLAGS) ${INC} $(PATH_TEST_SRC_UMQT) ${LINK} -o $(PATH_TEST_BIN) && $(PATH_TEST_BIN)

.PHONY: \
	build \
	test_rpc \
	test_umqt