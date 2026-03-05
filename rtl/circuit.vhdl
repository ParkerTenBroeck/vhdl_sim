library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Do not modify the following entity block
entity circuit is
port (
  clk: in std_logic; -- 500 Hz, period 2 ms
  key: in std_logic_vector(31 downto 0);   -- active low
  sw: in std_logic_vector(31 downto 0);   -- active high
  led: out std_logic_vector(31 downto 0) := (others => '0');  -- active high
  hex: out std_logic_vector(31 downto 0) := (others => '0')  -- active low
  );
end circuit;


architecture description of circuit is
  signal counter: unsigned(9 downto 0) := "0000000000";
begin
  led(9 downto 0) <= std_logic_vector(counter);
  -- led(10) <= clk;
  process(sw(0))
  begin
    counter <= counter+1;
    report "meow";
  end process;
end description;