library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;


entity tb is
  
end entity;

architecture sim of tb is
  signal clk  : std_logic := '0';
  signal btn  : std_logic_vector(31 downto 0) := (others => '0');
  signal sw   : std_logic_vector(31 downto 0) := (others => '0');
  signal led  : std_logic_vector(31 downto 0) := (others => '0');
  signal segv : std_logic_vector(31 downto 0) := (others => '0');
  signal segs : std_logic_vector(31 downto 0) := (others => '0');


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

  function ffi_get_btn return integer is
  begin
    return 0;
  end function;
  attribute foreign of ffi_get_btn : function is "VHPIDIRECT ffi_get_btn";

  procedure ffi_set_outputs(led_i: integer; segv_i: integer; segs_i: integer) is
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
      btn  => btn,
      sw   => sw,
      led  => led,
      segv => segv,
      segs => segs
    );

  -- 500 Hz clock (2 ms period)
  clk <= not clk after 1 ms;

  process
    variable sw_i  : integer;
    variable btn_i : integer;
  begin
    ffi_init;
    wait for 0 ns;
    
    while true loop
      wait until rising_edge(clk) or falling_edge(clk);
      wait for 0 ns;

      sw_i  := ffi_get_sw;
      btn_i := ffi_get_btn;

      sw  <= std_logic_vector(to_signed(sw_i, 32));
      btn <= std_logic_vector(to_signed(btn_i, 32));

      ffi_set_outputs(
          to_integer(signed(clean_slv(led))),
          to_integer(signed(clean_slv(segv))),
          to_integer(signed(clean_slv(segs)))
      );
    end loop;
  end process;

end architecture;