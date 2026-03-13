module bcd (
    input  wire              clk,
    input  wire signed [22:0] num,  // sfixed(15 downto -7)
    input  wire              en,
    output reg  [63:0]       seg
);

    function [7:0] seg_encode;
        input [3:0] d;
        begin
            case (d)
                4'd0: seg_encode = 8'b00111111;
                4'd1: seg_encode = 8'b00000110;
                4'd2: seg_encode = 8'b01011011;
                4'd3: seg_encode = 8'b01001111;
                4'd4: seg_encode = 8'b01100110;
                4'd5: seg_encode = 8'b01101101;
                4'd6: seg_encode = 8'b01111101;
                4'd7: seg_encode = 8'b00000111;
                4'd8: seg_encode = 8'b01111111;
                4'd9: seg_encode = 8'b01101111;
                default: seg_encode = 8'b00000000;
            endcase
        end
    endfunction

    function [3:0] to_bcd_digit;
        input integer value;
        integer remainder;
        begin
            remainder = value % 10;
            case (remainder)
                0: to_bcd_digit = 4'd0;
                1: to_bcd_digit = 4'd1;
                2: to_bcd_digit = 4'd2;
                3: to_bcd_digit = 4'd3;
                4: to_bcd_digit = 4'd4;
                5: to_bcd_digit = 4'd5;
                6: to_bcd_digit = 4'd6;
                7: to_bcd_digit = 4'd7;
                8: to_bcd_digit = 4'd8;
                9: to_bcd_digit = 4'd9;
                default: to_bcd_digit = 4'd0;
            endcase
        end
    endfunction

    integer scaled_hundredths;
    integer magnitude;
    integer frac_hundredths;
    integer tmp;
    integer j;
    reg negative;
    reg [3:0] digits [0:7];
    reg [63:0] out_seg;

    always @* begin
        if (!en) begin
            seg = 64'b0;
        end else begin
            out_seg = 64'b0;

            // num is Q16.7 fixed-point, so value = num / 128.
            // Round to nearest hundredth.
            if (num < 0) begin
                scaled_hundredths = ((num * 100) - 64) / 128;
            end else begin
                scaled_hundredths = ((num * 100) + 64) / 128;
            end

            negative = (scaled_hundredths < 0);

            if (negative) begin
                magnitude = -scaled_hundredths;
            end else begin
                magnitude = scaled_hundredths;
            end

            frac_hundredths = magnitude % 100;
            tmp = magnitude;

            for (j = 0; j < 8; j = j + 1) begin
                digits[j] = to_bcd_digit(tmp);
                tmp = tmp / 10;
            end

            for (j = 0; j < 7; j = j + 1) begin
                out_seg[(7 - j) * 8 +: 8] = seg_encode(digits[j]);
            end

            out_seg[5 * 8 + 7] = 1'b1;

            if (negative) begin
                out_seg[6] = 1'b1;
            end

            seg = out_seg;
        end
    end

endmodule
