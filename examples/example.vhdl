library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Do not modify the following entity block
entity circuit is
port (
  clk: in std_logic; -- 500 Hz, period 2 ms
  btn: in std_logic_vector(31 downto 0);
  sw: in std_logic_vector(31 downto 0);
  led: out std_logic_vector(31 downto 0) := (others => '0');
  segv: out std_logic_vector(31 downto 0);
  segs: out std_logic_vector(31 downto 0)
  );
end circuit;


architecture description of circuit is
  signal counter: unsigned(31 downto 0) := x"00000000";
begin
  led <= std_logic_vector(counter);
  process(clk)
  begin
    counter <= counter+1;
  end process;
end description;