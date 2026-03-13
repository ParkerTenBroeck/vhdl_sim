// Do not modify the following module interface.
module circuit (
    input  wire        clk,   // 500 Hz, period 2 ms
    input  wire [31:0] btn,
    input  wire [31:0] sw,
    output reg  [31:0] led = 32'h00000000,
    output wire [31:0] segv,
    output wire [31:0] segs
);
    reg [31:0] counter = 32'h00000000;

    assign segv = 32'h00000000;
    assign segs = 32'h00000000;

    always @(posedge clk) begin
        counter <= counter + 32'd1;
        led <= counter ^ sw ^ btn;
    end
endmodule
