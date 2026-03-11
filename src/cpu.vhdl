library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Do not modify the following entity block
entity circuit is
port (
  clk: in std_logic; 
  btn: in std_logic_vector(31 downto 0); 
  sw: in std_logic_vector(31 downto 0); 
  led: out std_logic_vector(31 downto 0);
  seg0: out std_logic_vector(31 downto 0);
  seg1: out std_logic_vector(31 downto 0);
  seg2: out std_logic_vector(31 downto 0);
  seg3: out std_logic_vector(31 downto 0)
  );
end circuit;


library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;


architecture description of circuit is
  signal clock: std_logic;

  signal reg_pc: unsigned(7 downto 0) := "00000000";
  signal reg_a: unsigned(7 downto 0) := "00000000";
  signal reg_b: unsigned(7 downto 0) := "00000000";
  signal reg_out: unsigned(7 downto 0) := "00000000";

  signal inst_reg: unsigned(7 downto 0);
  signal inst_bus: unsigned(7 downto 0);

  signal data_read: unsigned(7 downto 0);
  signal data_write: unsigned(7 downto 0) := "00000000";

  signal data_addr: unsigned(7 downto 0) := "00000000";
  signal data_write_e: std_logic := '0';

  signal flag_carry: std_logic := '0';
  signal flag_lt: std_logic := '0';
  signal flag_gt: std_logic := '0';
  signal flag_eq: std_logic := '0';
  signal flag_zero: std_logic := '0';

  function dec7seg(val: unsigned(3 downto 0)) return std_logic_vector is
    begin
      case val is
        when "0000"=> return "0111111"; --0
        when "0001"=> return "0000110"; --1
        when "0010"=> return "1011011"; --2
        when "0011"=> return "1001111"; --3
        when "0100"=> return "1100110"; --4
        when "0101"=> return "1101101"; --5
        when "0110"=> return "1111101"; --6
        when "0111"=> return "0000111"; --7
        when "1000"=> return "1111111"; --8
        when "1001"=> return "1100111"; --9
        when "1010"=> return "1110111"; --A
        when "1011"=> return "1111100"; --B
        when "1100"=> return "0111001"; --C
        when "1101"=> return "1011110"; --D
        when "1110"=> return "1111001"; --E
        when "1111"=> return "1110001"; --F
        when others=> return "0000000"; ---
      end case;
  end function;
  

begin

  seg0(6 downto 0) <= dec7seg(reg_out(7 downto 4));
  seg0(14 downto 8) <= dec7seg(reg_out(3 downto 0));

  seg0(22 downto 16) <= dec7seg(reg_pc(7 downto 4));
  seg0(30 downto 24) <= dec7seg(reg_pc(3 downto 0));

  led(7 downto 0) <= std_logic_vector(reg_a);
  led(23 downto 16) <= std_logic_vector(reg_b);

  led(8) <= flag_zero;
  led(9) <= flag_eq;
  led(10) <= flag_lt;
  led(11) <= flag_gt;
  led(12) <= flag_carry;

  clock <= clk when sw(9) = '1' else btn(0);
  
  ram_inst : entity work.inst_ram_8x256
    port map(
        clk  => clk,
        addr => reg_pc, 
        dout => inst_bus
    );

  ram_data : entity work.ram_8x256
    port map(
        clk  => clk,
        we   => data_write_e,
        addr => data_addr,
        din  => data_write,
        dout => data_read
    );



  process(clock)
    variable alu_tmp: unsigned(8 downto 0) := "000000000";
    variable alu_a: unsigned(7 downto 0) := "00000000";
    variable alu_b: unsigned(7 downto 0) := "00000000";
    variable branch: std_logic := '0';
  begin

    if rising_edge(clock) then
      inst_reg <= inst_bus;
      data_write_e <= '0';

      --report "begin reg_a = " & integer'image(to_integer(unsigned(reg_a)))
      -- & " reg_b = " & integer'image(to_integer(unsigned(reg_b)))
      -- & " reg_out = " & integer'image(to_integer(unsigned(reg_out)))
      -- & " reg_pc = " & integer'image(to_integer(unsigned(reg_pc)))
      -- & " inst_bus = " & integer'image(to_integer(unsigned(inst_bus)));

      case to_integer(inst_bus) is
        -- nop
        when 16#00# => null;
        -- a, b swap
        when 16#01# => 
          reg_a <= reg_b;
          reg_b <= reg_a;

        -- alu operations a,imm
        -- alu operations imm,b
        when 16#20# to 16#3F# => reg_pc <= reg_pc+1;


        -- 0 => a
        when 16#A0# => reg_a <= x"00";
        -- 1 => a
        when 16#A1# => reg_a <= x"01";
        -- mem[reg b] => a 
        when 16#AC# =>
         data_addr <= reg_b;
        -- out => a
        when 16#AE# => reg_a <= reg_out;
        -- immediate => a
        when 16#AF# => reg_pc <= reg_pc+1;

        -- 0 => b
        when 16#B0# => reg_b <= x"00";
        -- 1 => b
        when 16#B1# => reg_b <= x"01";
        -- mem[reg a] => b 
        when 16#BC# => 
         data_addr <= reg_b;
        -- out => b
        when 16#BE# => reg_b <= reg_out;
        -- immediate => b
        when 16#BF# => reg_pc <= reg_pc+1;

        -- conditional jump
        when 16#C0# to 16#CF# => reg_pc <= reg_pc+1;

        -- jump imm addr abs
        when 16#D0# => reg_pc <= reg_pc+1;
        -- jump imm addr rel
        when 16#D1# => reg_pc <= reg_pc+1;
        -- jump addr reg a
        when 16#DA# => reg_pc <= reg_a-1;
        -- jump addr reg b
        when 16#DB# => reg_pc <= reg_b-1;

        -- out
        when 16#FE# => report " out = " & integer'image(to_integer(unsigned(reg_out)));
        -- halt
        when 16#FF# => reg_pc <= reg_pc-1;

        when others =>  null;
      end case;

      case to_integer(inst_bus(2 downto 0)) is
        when 0 => branch := flag_zero;
        when 1 => branch := flag_carry;
        when 2 => branch := flag_eq;
        when 3 => branch := not flag_eq;
        when 4 => branch := flag_lt;
        when 5 => branch := flag_gt;
        when 6 => branch := flag_lt or flag_eq;
        when 7 => branch := flag_gt or flag_eq;
        when others =>  null;
      end case;
    end if;

    if falling_edge(clock) then
      case to_integer(inst_reg) is
        when 16#AC# => reg_a <= data_read;
        when 16#AF# => reg_a <= inst_bus;

        when 16#BC# => reg_b <= data_read;
        when 16#BF# => reg_b <= inst_bus;

      -- alu operation a,b
        when 16#10# to 16#1F# => 
          alu_a := reg_a;
          alu_b := reg_b;

      -- alu operations a,imm
        when 16#20# to 16#2F# => 
          alu_a := reg_a;
          alu_b := inst_bus;

      -- alu operations imm,b
        when 16#30# to 16#3F# => 
          alu_a := inst_bus;
          alu_b := reg_b; 

        when 16#C0# to 16#C7# => alu_tmp(7 downto 0) := inst_bus;
        when 16#C8# to 16#CF# => alu_tmp(7 downto 0) := inst_bus+reg_pc;

        when others => null;
      end case;

      -- alu operation
      if inst_reg(7 downto 4) = x"1" or inst_reg(7 downto 4) = x"2" or inst_reg(7 downto 4) = x"3" then
          with inst_reg(3 downto 0) select
            alu_tmp :=  ("0"&alu_a) + ("0"&alu_b) when x"0",
                  ("0"&alu_a) + ("0"&alu_b) + (x"00"&flag_carry) when x"1",
                  ("0"&alu_a) - ("0"&alu_b) when x"2",
                  ("0"&alu_a) - ("0"&alu_b) - (x"00"&flag_carry) when x"3",
                  ("0"&alu_a) and ("0"&alu_b) when x"4",
                  ("0"&alu_a) or ("0"&alu_b) when x"5",
                  ("0"&alu_a) xor ("0"&alu_b) when x"6",
                  shift_left("0"&alu_a,to_integer("0"&alu_b)) when x"7",
                  shift_right("0"&alu_a,to_integer("0"&alu_b)) when x"8",
                  rotate_left("0"&alu_a,to_integer("0"&alu_b)) when x"9",
                  rotate_right("0"&alu_a,to_integer("0"&alu_b)) when x"A",
                  "0"&x"00" when others;

          flag_zero <= '1' when alu_tmp = 0 else '0';
          flag_eq <= '1' when alu_a = alu_b else '0';
          flag_lt <= '1' when alu_a < alu_b else '0';
          flag_gt <= '1' when alu_a > alu_b else '0';
          flag_carry <= alu_tmp(8); 
          reg_out <= alu_tmp(7 downto 0);

      end if;


      case to_integer(inst_reg) is
        -- jump imm addr abs
        when 16#D0# => reg_pc <= inst_bus;
        -- jump imm addr rel
        when 16#D1# => reg_pc <= reg_pc+inst_bus;

        -- conditional brances
        when 16#C0# to 16#CF# => 
          if branch then 
            reg_pc <= alu_tmp(7 downto 0); 
          else 
            reg_pc <= reg_pc+1;
          end if;

        when others => reg_pc <= reg_pc+1;
      end case;

      -- report "end reg_a = " & integer'image(to_integer(unsigned(reg_a)))
      --   & " reg_b = " & integer'image(to_integer(unsigned(reg_b)))
      --   & " reg_out = " & integer'image(to_integer(unsigned(reg_out)))
      --   & " reg_pc = " & integer'image(to_integer(unsigned(reg_pc)))
      --   & " inst_bus = " & integer'image(to_integer(unsigned(inst_bus)));
      

    end if;
    
  end process;
end description;