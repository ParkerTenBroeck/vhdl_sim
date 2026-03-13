#include <cstdint>
#include "Vcircuit.h"
#include "verilated.h"

extern "C" void ffi_init();
extern "C" std::uint32_t ffi_get_sw();
extern "C" std::uint32_t ffi_get_btn();
extern "C" void ffi_set_outputs(std::uint32_t led, std::uint32_t segv, std::uint32_t segs);

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);

    Vcircuit top;
    top.clk = 0;
    top.btn = 0;
    top.sw = 0;

    ffi_init();

    while (true) {
        top.sw = ffi_get_sw();
        top.btn = ffi_get_btn();

        top.clk = 0;
        top.eval();
        ffi_set_outputs(top.led, top.segv, top.segs);

        top.sw = ffi_get_sw();
        top.btn = ffi_get_btn();

        top.clk = 1;
        top.eval();
        ffi_set_outputs(top.led, top.segv, top.segs);
    }
}