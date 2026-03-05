library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Do not modify the following entity block
entity circuit is
port (
  clk: in std_logic; 
  key: in std_logic_vector(31 downto 0);   -- active low
  sw: in std_logic_vector(31 downto 0);   -- active high
  led: out std_logic_vector(31 downto 0);  -- active high
  hex: out std_logic_vector(31 downto 0)  -- active low
  );
end circuit;


library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity alu is
  Port (
    func: in unsigned(3 downto 0);
    a: in unsigned(7 downto 0);
    b: in unsigned(7 downto 0);
    carry_in: in std_logic;
    o: out unsigned(7 downto 0);
    carry_out: out std_logic;
    zero: out std_logic;
    gt: out std_logic;
    lt: out std_logic;
    eq: out std_logic
  );
end alu;

architecture Behavioral of alu is
    signal tmp: unsigned(8 downto 0);
begin
  with func select
    tmp <=  ("0"&a) + ("0"&b) when x"0",
          ("0"&a) + ("0"&b) + (x"00"&carry_in) when x"1",
          ("0"&a) - ("0"&b) when x"2",
          ("0"&a) - ("0"&b) - (x"00"&carry_in) when x"3",
          ("0"&a) and ("0"&b) when x"4",
          ("0"&a) or ("0"&b) when x"5",
          ("0"&a) xor ("0"&b) when x"6",
          "0"&x"00" when others;

  zero <= '1' when tmp = 0 else '0';
  eq <= '1' when a = b else '0';
  lt <= '1' when a < b else '0';
  gt <= '1' when a > b else '0';
  carry_out <= tmp(8); 
  o <= tmp(7 downto 0);

end Behavioral ; -- Behavioral

library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity ram_8x256 is
    Port (
        clk   : in  std_logic;
        we    : in  std_logic;  -- write enable
        addr  : in  unsigned(7 downto 0); -- 8-bit address
        din   : in  unsigned(7 downto 0); -- data input
        dout  : out unsigned(7 downto 0)  -- data output
    );
end ram_8x256;

architecture Behavioral of ram_8x256 is
    type ram_type is array (0 to 255) of unsigned(7 downto 0);
    signal ram : ram_type := (others => x"AB");
begin
    process(clk)
    begin
        if rising_edge(clk) then
            if we = '1' then
                ram(to_integer(unsigned(addr))) <= din;
            end if;

            dout <= ram(to_integer(unsigned(addr)));
        end if;
    end process;
end Behavioral;


library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity inst_ram_8x256 is
    Port (
        clk   : in  std_logic;
        addr  : in  unsigned(7 downto 0); -- 8-bit address
        dout  : out unsigned(7 downto 0)  -- data output
    );
end inst_ram_8x256;

architecture Behavioral of inst_ram_8x256 is
    type ram_type is array (0 to 255) of unsigned(7 downto 0);
    signal ram : ram_type := (
      0 => x"A0", -- 0 => a
      2 => x"B1", -- 1 => b
      3 => x"10", -- a+b => out
      4 => x"AE", -- out => a
      5 => x"10", -- a+b => out
      6 => x"BE", -- out => b
      7 => x"D0", -- jump to 3
      8 => x"03",
      others => (others => '0')
    );
begin
    process(clk)
    begin
        if rising_edge(clk) or falling_edge(clk) then
            dout <= ram(to_integer(unsigned(addr)));
        end if;
    end process;
end Behavioral;


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
  signal data_write: unsigned(7 downto 0);

  signal data_addr: unsigned(7 downto 0) := "00000000";
  signal data_write_e: std_logic := '0';

  signal flag_carry: std_logic := '0';
  signal flag_lt: std_logic := '0';
  signal flag_gt: std_logic := '0';
  signal flag_eq: std_logic := '0';
  signal flag_zero: std_logic := '0';

  signal alu_a, alu_b, alu_o : unsigned(7 downto 0);
  signal alu_func : unsigned(3 downto 0);
  signal alu_carry, alu_zero, alu_gt, alu_lt, alu_eq : std_logic;

  function dec7seg(val: unsigned(3 downto 0)) return std_logic_vector is
    begin
      case val is
        when "0000"=> return "1000000"; --0
        when "0001"=> return "1111001"; --1
        when "0010"=> return "0100100"; --2
        when "0011"=> return "0110000"; --3
        when "0100"=> return "0011001"; --4
        when "0101"=> return "0010010"; --5
        when "0110"=> return "0000010"; --6
        when "0111"=> return "1111000"; --7
        when "1000"=> return "0000000"; --8
        when "1001"=> return "0011000"; --9
        when "1010"=> return "0001000"; --A
        when "1011"=> return "0000011"; --B
        when "1100"=> return "1000110"; --C
        when "1101"=> return "0100001"; --D
        when "1110"=> return "0000110"; --E
        when "1111"=> return "0001110"; --F
        when others=> return "1111111"; ---
      end case;
  end function;
  

begin

  -- hex(7 downto 4) <= dec7seg(reg_out(7 downto 4));
  -- hex(3 downto 0) <= dec7seg(reg_out(3 downto 0));

  clock <= clk when sw(9) = '1' else sw(8);
  
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

    alu : entity work.alu
      port map(
        func      => alu_func,
        a         => alu_a,
        b         => alu_b,
        carry_in  => flag_carry,
        o         => alu_o,
        carry_out => alu_carry,
        zero      => alu_zero,
        gt        => alu_gt,
        lt        => alu_lt,
        eq        => alu_eq
      );

  process(clock)
    variable out_extended : unsigned(8 downto 0);
  begin

    if rising_edge(clock) then
      inst_reg <= inst_bus;
      data_write_e <= '0';

      report "begin reg_a = " & integer'image(to_integer(unsigned(reg_a)))
      & " reg_b = " & integer'image(to_integer(unsigned(reg_b)))
      & " reg_out = " & integer'image(to_integer(unsigned(reg_out)))
      & " reg_pc = " & integer'image(to_integer(unsigned(reg_pc)))
      & " inst_bus = " & integer'image(to_integer(unsigned(inst_bus)));

      -- alu operations a,b
      if inst_reg(7 downto 4) = x"1" then
        alu_func <= inst_reg(3 downto 0);
        alu_a <= reg_a;
        alu_b <= reg_b;
        reg_out <= alu_o;
      end if;
      -- alu operations a,imm
      if inst_reg(7 downto 4) = x"2" then
        alu_func <= inst_reg(3 downto 0);
        alu_a <= reg_a;
        reg_pc <= reg_pc+1;
      end if;
      -- alu operations imm,b
      if inst_reg(7 downto 4) = x"3" then
        alu_func <= inst_reg(3 downto 0);
        alu_b <= reg_b;
        reg_pc <= reg_pc+1;
      end if;

      case inst_bus is
        -- nop
        when x"00" => null;


        -- 0 => a
        when x"A0" => reg_a <= x"00";
        -- 1 => a
        when x"A1" => reg_a <= x"01";
        -- mem[reg b] => a 
        when x"AC" =>
         data_addr <= reg_b;
        -- out => a
        when x"AE" => reg_a <= reg_out;
        -- immediate => a
        when x"AF" => reg_pc <= reg_pc+1;

        -- 0 => b
        when x"B0" => reg_b <= x"00";
        -- 1 => b
        when x"B1" => reg_b <= x"01";
        -- mem[reg a] => b 
        when x"BC" => 
         data_addr <= reg_b;
        -- out => b
        when x"BE" => reg_b <= reg_out;
        -- immediate => b
        when x"BF" => reg_pc <= reg_pc+1;

        -- conditional

        -- jump imm addr abs
        when x"D0" => reg_pc <= reg_pc+1;
        -- jump imm addr rel
        when x"D1" => reg_pc <= reg_pc+1;
        -- jump addr reg a
        when x"DA" => reg_pc <= reg_a-1;
        -- jump addr reg b
        when x"DB" => reg_pc <= reg_b-1;

        -- halt
        when x"FF" => reg_pc <= reg_pc-1;

        when others =>  null;
      end case;
    end if;

    if falling_edge(clock) then
      case inst_reg is
        when x"AC" => reg_a <= data_read;
        when x"AF" => reg_a <= inst_bus;

        when x"BC" => reg_b <= data_read;
        when x"BF" => reg_b <= inst_bus;

        when others => null;
      end case;


      case inst_reg is
        -- jump imm addr abs
        when x"D0" => reg_pc <= inst_bus;
        -- jump imm addr rel
        when x"D1" => reg_pc <= reg_pc+inst_bus;

        when others => reg_pc <= reg_pc+1;
      end case;

            report "end reg_a = " & integer'image(to_integer(unsigned(reg_a)))
      & " reg_b = " & integer'image(to_integer(unsigned(reg_b)))
      & " reg_out = " & integer'image(to_integer(unsigned(reg_out)))
      & " reg_pc = " & integer'image(to_integer(unsigned(reg_pc)))
      & " inst_bus = " & integer'image(to_integer(unsigned(inst_bus)));
      

    end if;
    
  end process;
end description;