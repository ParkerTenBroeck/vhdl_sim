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
