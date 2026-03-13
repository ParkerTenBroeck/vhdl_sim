module seg_plex (
    input  wire        clk,

    input  wire [63:0] seg0,
    input  wire [63:0] seg1,
    input  wire [63:0] seg2,
    input  wire [63:0] seg3,

    output reg  [31:0] segv,
    output reg  [2:0]  segs
);

    reg [2:0] counter = 3'b000;

    always @(posedge clk) begin
        case (counter)
            3'd0: segv <= seg0[31:0];
            3'd1: segv <= seg0[63:32];
            3'd2: segv <= seg1[31:0];
            3'd3: segv <= seg1[63:32];
            3'd4: segv <= seg2[31:0];
            3'd5: segv <= seg2[63:32];
            3'd6: segv <= seg3[31:0];
            3'd7: segv <= seg3[63:32];
            default: segv <= 32'b0;
        endcase

        counter <= counter + 3'd1;
        segs <= counter;
    end

endmodule
