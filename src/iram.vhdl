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
      1 => x"B1", -- 1 => b
      2 => x"10", -- a+b => out
      3 => x"FE", -- out
      4 => x"AE", -- out => a
      5 => x"01", -- swap a/b
      
      6 => x"3F", -- cmp 144, b
      7 => x"90",
      
      8 => x"C7", -- jump to 2 if 144 <= b
      9 => x"02",
      
      10 => x"FF", -- halt
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
