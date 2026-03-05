library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity tb is
end entity;

architecture sim of tb is
  signal clk  : std_logic := '0';
  signal key  : std_logic_vector(31 downto 0) := (others => '1'); -- active low
  signal sw   : std_logic_vector(31 downto 0) := (others => '0');

  signal led  : std_logic_vector(31 downto 0) := (others => '0');
  signal hex : std_logic_vector(31 downto 0) := (others => '0');


  -- Foreign subprograms MUST be declared in the declarative region (here),
  -- and MUST have a body (even dummy) to satisfy VHDL.
  procedure ffi_init is
  begin
  end procedure;
  attribute foreign of ffi_init : procedure is
    "VHPIDIRECT ffi_init";

  function ffi_get_sw return integer is
  begin
    return 0;
  end function;
  attribute foreign of ffi_get_sw : function is
    "VHPIDIRECT ffi_get_sw";

  function ffi_get_key return integer is
  begin
    return 0;
  end function;
  attribute foreign of ffi_get_key : function is "VHPIDIRECT ffi_get_key";

  procedure ffi_set_outputs(led_i : integer; hex_i : integer) is
  begin
  end procedure;
  attribute foreign of ffi_set_outputs : procedure is
    "VHPIDIRECT ffi_set_outputs";

  function clean_slv(v : std_logic_vector) return std_logic_vector is
    variable r : std_logic_vector(v'range);
  begin
    for i in v'range loop
      if v(i) = '1' then
        r(i) := '1';
      else
        r(i) := '0';
      end if;
    end loop;
    return r;
  end function;

begin
  dut: entity work.circuit
    port map (
      clk  => clk,
      key  => key,
      sw   => sw,
      led  => led,
      hex => hex
    );

  -- 500 Hz clock (2 ms period)
  clk <= not clk after 1 ms;

  process
    variable sw_i  : integer;
    variable key_i : integer;
  begin
    ffi_init;
    wait for 0 ns;
    
    while true loop
      wait until rising_edge(clk) or falling_edge(clk);
      wait for 0 ns;

      sw_i  := ffi_get_sw;
      key_i := ffi_get_key;

      sw  <= std_logic_vector(to_signed(sw_i, 32));
      key <= std_logic_vector(to_signed(key_i, 32));

      ffi_set_outputs(
          to_integer(unsigned(clean_slv(led))),
          to_integer(unsigned(clean_slv(hex)))
      );
    end loop;
  end process;

end architecture;