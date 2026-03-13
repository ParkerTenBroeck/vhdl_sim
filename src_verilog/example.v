// Do not modify the following module interface.
module circuit (
    input  wire        clk,   // 500 Hz, period 2 ms
    input  wire [31:0] btn,
    input  wire [31:0] sw,
    output wire [31:0] led,
    output wire [31:0] segv,
    output wire [31:0] segs
);

    wire [3:0] dig;
    wire       dig_e;
    wire       dot;
    wire       eq;
    wire [3:0] op;
    wire       op_e;

    wire [63:0] seg_a;
    wire [2:0]  segs_mux;
    wire signed [22:0] sw_fixed;

    assign led = 32'b0;
    assign segs = {29'b0, segs_mux};

    // Convert signed 16-bit integer to signed Q16.7 fixed-point.
    assign sw_fixed = {sw[15:0], 7'b0};

    keypad_input keypad_input_inst (
        .clk(clk),
        .keypad(btn[15:8]),
        .dig(dig),
        .dig_e(dig_e),
        .dot(dot),
        .eq(eq),
        .op(op),
        .op_e(op_e)
    );

    bcd bcd_inst (
        .clk(clk),
        .num(sw_fixed),
        .en(1'b1),
        .seg(seg_a)
    );

    seg_plex seg_plex_inst (
        .clk(clk),
        .seg0(seg_a),
        .seg1(64'h0000000000000000),
        .seg2(64'h0000000000000000),
        .seg3(64'h0000000000000000),
        .segv(segv),
        .segs(segs_mux)
    );

endmodule
