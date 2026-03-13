module keypad_input (
    input  wire       clk,
    input  wire [7:0] keypad,

    output reg  [3:0] dig,
    output reg        dig_e,

    output reg        dot,
    output reg        eq,

    output reg  [3:0] op,
    output reg        op_e
);

    reg [7:0] keypad_curr = 8'b00000000;

    always @(posedge clk or negedge clk) begin
        if (clk && (keypad != keypad_curr)) begin
            case (keypad)
                8'b10001000: begin op <= 4'b0001; op_e <= 1'b1; end
                8'b10000100: begin eq <= 1'b1; end
                8'b10000010: begin dot <= 1'b1; end
                8'b10000001: begin dig <= 4'b0000; dig_e <= 1'b1; end

                8'b01001000: begin op <= 4'b0010; op_e <= 1'b1; end
                8'b01000100: begin dig <= 4'b0011; dig_e <= 1'b1; end
                8'b01000010: begin dig <= 4'b0010; dig_e <= 1'b1; end
                8'b01000001: begin dig <= 4'b0001; dig_e <= 1'b1; end

                8'b00101000: begin op <= 4'b0011; op_e <= 1'b1; end
                8'b00100100: begin dig <= 4'b0110; dig_e <= 1'b1; end
                8'b00100010: begin dig <= 4'b0101; dig_e <= 1'b1; end
                8'b00100001: begin dig <= 4'b0100; dig_e <= 1'b1; end

                8'b00011000: begin op <= 4'b0100; op_e <= 1'b1; end
                8'b00010100: begin dig <= 4'b1000; dig_e <= 1'b1; end
                8'b00010010: begin dig <= 4'b1000; dig_e <= 1'b1; end
                8'b00010001: begin dig <= 4'b0111; dig_e <= 1'b1; end

                default: begin end
            endcase
            keypad_curr <= keypad;
        end

        if (!clk && (keypad != keypad_curr)) begin
            dig   <= 4'b0000;
            dig_e <= 1'b0;
            eq    <= 1'b0;
            dot   <= 1'b0;
            op    <= 4'b0000;
            op_e  <= 1'b0;
        end
    end

endmodule
