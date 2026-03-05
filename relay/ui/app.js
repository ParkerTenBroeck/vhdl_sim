// ----- LocalStorage keys -----
const LS_KEY_VHDL = "circuit_ui:circuit.vhdl";

// ----- State -----
let ws = null;
let isConnected = false;

let switches = 0 >>> 0; // u32
let buttons  = 0 >>> 0; // u32

let autoscroll = true;

// ----- DOM -----
const wsPill = document.getElementById("wsPill");
const wsUrlText = document.getElementById("wsUrlText");

const connectToggleBtn = document.getElementById("connectToggleBtn");
const sendInputsBtn = document.getElementById("sendInputsBtn");

const vhdlEditor = document.getElementById("vhdlEditor");
const lineGutter = document.getElementById("lineGutter");
const loadExampleBtn = document.getElementById("loadExampleBtn");

const ledRow = document.getElementById("ledRow");
const ledLabels = document.getElementById("ledLabels");
const hexRow = document.getElementById("hexRow");

const switchGrid = document.getElementById("switchGrid");
const buttonGrid = document.getElementById("buttonGrid");

const logView = document.getElementById("logView");
const clearLogBtn = document.getElementById("clearLogBtn");
const autoscrollBtn = document.getElementById("autoscrollBtn");

const allSwOffBtn = document.getElementById("allSwOffBtn");
const allSwOnBtn = document.getElementById("allSwOnBtn");

// ----- Helpers -----
function wsUrl() {
  const proto = (location.protocol === "https:") ? "wss" : "ws";
  return `${proto}://${location.host}/ws`;
}

function setStatus(connected) {
  isConnected = connected;
  wsPill.textContent = connected ? "CONNECTED" : "DISCONNECTED";
  wsPill.style.borderColor = connected ? "rgba(34,197,94,.6)" : "rgba(239,68,68,.6)";
  wsPill.style.background = connected ? "rgba(34,197,94,.14)" : "rgba(239,68,68,.10)";

  sendInputsBtn.disabled = !connected;

  // single button label
  connectToggleBtn.textContent = connected ? "Disconnect" : "Connect";
  connectToggleBtn.classList.toggle("secondary", connected);
}

function appendLog(stream, line) {
  const prefix = stream === "stderr" ? "[stderr]" : "[stdout]";
  logView.textContent += `${prefix} ${line}\n`;
  if (autoscroll) {
    logView.scrollTop = logView.scrollHeight;
  }
}

function clearLogs() {
  logView.textContent = "";
}

function resetOutputsVisuals() {
  // reset LED/HEX visuals to 0 immediately
  setLeds(0);
  setHex(0);
}

function u32BitGet(x, i) {
  return ((x >>> i) & 1) === 1;
}

function u32BitSet(x, i, on) {
  if (on) return (x | (1 << i)) >>> 0;
  return (x & ~(1 << i)) >>> 0;
}

function sendClientInput() {
  if (!ws || ws.readyState !== WebSocket.OPEN) return;

  const msg = {
    type: "client_input",
    switch: switches >>> 0,
    buttons: buttons >>> 0,
  };
  ws.send(JSON.stringify(msg));
}

function parseHexDigits(hexU32) {
  const d0 = (hexU32 >>> 0) & 0xFF;
  const d1 = (hexU32 >>> 8) & 0xFF;
  const d2 = (hexU32 >>> 16) & 0xFF;
  const d3 = (hexU32 >>> 24) & 0xFF;
  return [d0, d1, d2, d3];
}

// ----- Line numbers -----
function updateLineNumbers() {
  const text = vhdlEditor.value || "";
  // count lines: number of '\n' + 1 (even empty text -> 1 line)
  const lines = text.length ? (text.split("\n").length) : 1;

  // Build as one string for performance
  let out = "";
  for (let i = 1; i <= lines; i++) out += i + "\n";
  lineGutter.textContent = out;

  syncGutterScroll();
}

function syncGutterScroll() {
  // keep gutter aligned to editor scroll
  lineGutter.scrollTop = vhdlEditor.scrollTop;
}

// ----- LocalStorage save (debounced) -----
let saveTimer = null;
function saveEditorToLocalStorageDebounced() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    try {
      localStorage.setItem(LS_KEY_VHDL, vhdlEditor.value ?? "");
    } catch (e) {
      // ignore storage failures (private mode, quota)
    }
  }, 250);
}

function loadEditorFromLocalStorage() {
  try {
    const saved = localStorage.getItem(LS_KEY_VHDL);
    if (saved !== null) return saved;
  } catch {}
  return null;
}

// ----- UI Builders -----
function buildLeds() {
  ledRow.innerHTML = "";
  ledLabels.innerHTML = "";

  for (let i = 0; i < 32; i++) {
    const el = document.createElement("div");
    el.className = "led";
    el.title = `LED[${i}]`;
    el.dataset.bit = String(i);
    ledRow.appendChild(el);

    const lab = document.createElement("span");
    lab.textContent = String(i);
    ledLabels.appendChild(lab);
  }
}

function setLeds(bitsU32) {
  for (const el of ledRow.children) {
    const i = Number(el.dataset.bit);
    el.classList.toggle("on", u32BitGet(bitsU32 >>> 0, i));
  }
}

function makeSevenSeg(digitIndex) {
  const wrap = document.createElement("div");

  const disp = document.createElement("div");
  disp.className = "sevenSeg";
  disp.dataset.digit = String(digitIndex);

  const segs = ["a","b","c","d","e","f","g","dp"];
  for (const s of segs) {
    const seg = document.createElement("div");
    seg.className = `seg ${s}`;
    seg.dataset.seg = s;
    disp.appendChild(seg);
  }

  const label = document.createElement("div");
  label.className = "digitLabel";
  label.textContent = `HEX[${digitIndex}]`;

  wrap.appendChild(disp);
  wrap.appendChild(label);
  return wrap;
}

function buildHex() {
  hexRow.innerHTML = "";
  for (let i = 0; i < 4; i++) {
    hexRow.appendChild(makeSevenSeg(i));
  }
}

function setHex(hexU32) {
  const digits = parseHexDigits(hexU32 >>> 0);

  for (let i = 0; i < 4; i++) {
    const byte = digits[i] & 0xFF;
    const disp = hexRow.querySelector(`.sevenSeg[data-digit="${i}"]`);
    if (!disp) continue;

    const map = { a:0, b:1, c:2, d:3, e:4, f:5, g:6, dp:7 };

    for (const segEl of disp.querySelectorAll(".seg")) {
      const name = segEl.dataset.seg;
      const bit = map[name];
      const on = ((byte >>> bit) & 1) === 1;
      segEl.classList.toggle("on", on);
    }
  }
}

function buildSwitches() {
  switchGrid.innerHTML = "";
  for (let i = 0; i < 32; i++) {
    const cell = document.createElement("div");
    cell.className = "ioCell";

    const label = document.createElement("label");
    label.textContent = `SW[${i}]`;

    const toggle = document.createElement("div");
    toggle.className = "toggle";
    toggle.dataset.bit = String(i);
    toggle.title = `Toggle switch ${i}`;

    toggle.addEventListener("click", () => {
      const bit = Number(toggle.dataset.bit);
      const now = !u32BitGet(switches, bit);
      switches = u32BitSet(switches, bit, now);
      toggle.classList.toggle("on", now);
      sendClientInput();
    });

    cell.appendChild(toggle);
    cell.appendChild(label);
    switchGrid.appendChild(cell);
  }
}

function syncSwitchesUI() {
  for (const toggle of switchGrid.querySelectorAll(".toggle")) {
    const i = Number(toggle.dataset.bit);
    toggle.classList.toggle("on", u32BitGet(switches, i));
  }
}

function buildButtons() {
  buttonGrid.innerHTML = "";
  for (let i = 0; i < 32; i++) {
    const cell = document.createElement("div");
    cell.className = "ioCell";

    const label = document.createElement("label");
    label.textContent = `KEY[${i}]`;

    const btn = document.createElement("button");
    btn.className = "momentary";
    btn.type = "button";
    btn.textContent = "press";
    btn.dataset.bit = String(i);
    btn.title = `Momentary button ${i}`;

    const press = () => {
      const bit = Number(btn.dataset.bit);
      buttons = u32BitSet(buttons, bit, true);
      btn.classList.add("down");
      sendClientInput();
    };

    const release = () => {
      const bit = Number(btn.dataset.bit);
      buttons = u32BitSet(buttons, bit, false);
      btn.classList.remove("down");
      sendClientInput();
    };

    // Mouse
    btn.addEventListener("mousedown", (e) => { e.preventDefault(); press(); });
    btn.addEventListener("mouseup",   (e) => { e.preventDefault(); release(); });
    btn.addEventListener("mouseleave",(e) => { e.preventDefault(); release(); });

    // Touch
    btn.addEventListener("touchstart",(e) => { e.preventDefault(); press(); }, {passive:false});
    btn.addEventListener("touchend",  (e) => { e.preventDefault(); release(); }, {passive:false});
    btn.addEventListener("touchcancel",(e)=> { e.preventDefault(); release(); }, {passive:false});

    cell.appendChild(btn);
    cell.appendChild(label);
    buttonGrid.appendChild(cell);
  }
}

// ----- WebSocket -----
function connect() {
  if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

  // Requirement (3): reset logs + outputs when connect is pressed
  clearLogs();
  resetOutputsVisuals();

  const url = wsUrl();
  wsUrlText.textContent = url;

  ws = new WebSocket(url);

  ws.addEventListener("open", () => {
    setStatus(true);
    appendLog("stdout", "WebSocket connected.");

    // First message MUST be the file map
    const files = {
      "circuit.vhdl": vhdlEditor.value ?? ""
    };
    ws.send(JSON.stringify(files));

    // Push initial input state
    sendClientInput();
  });

  ws.addEventListener("message", (ev) => {
    let parsed = null;
    try {
      parsed = JSON.parse(ev.data);
    } catch {
      appendLog("stderr", String(ev.data));
      return;
    }


    if (parsed.log !== undefined) {
      appendLog(parsed.log.stream ?? "stdout", parsed.log.line ?? "");
      return;
    }

    if (parsed.led !== undefined) {
      const v = (parsed.led ?? parsed.value ?? parsed[0] ?? parsed["0"] ?? 0) >>> 0;
      setLeds(v);
      return;
    }

    if (parsed.hex !== undefined) {
      const v = (parsed.hex ?? parsed.value ?? parsed[0] ?? parsed["0"] ?? 0) >>> 0;
      setHex(v);
      return;
    }

    appendLog("stderr", `Unknown msg: ${ev.data}`);
  });

  ws.addEventListener("close", () => {
    appendLog("stderr", "WebSocket closed.");
    setStatus(false);
  });

  ws.addEventListener("error", () => {
    appendLog("stderr", "WebSocket error.");
    setStatus(false);
  });
}

function disconnect() {
  if (ws) ws.close();
}

function toggleConnect() {
  if (isConnected) disconnect();
  else connect();
}

// ----- Wire up controls -----
connectToggleBtn.addEventListener("click", toggleConnect);

sendInputsBtn.addEventListener("click", () => {
  appendLog("stdout", `Manual send: sw=${switches >>> 0} key=${buttons >>> 0}`);
  sendClientInput();
});

clearLogBtn.addEventListener("click", () => { clearLogs(); });

autoscrollBtn.addEventListener("click", () => {
  autoscroll = !autoscroll;
  autoscrollBtn.textContent = `Autoscroll: ${autoscroll ? "on" : "off"}`;
});

allSwOffBtn.addEventListener("click", () => {
  switches = 0 >>> 0;
  syncSwitchesUI();
  sendClientInput();
});

allSwOnBtn.addEventListener("click", () => {
  switches = 0xFFFF_FFFF >>> 0;
  syncSwitchesUI();
  sendClientInput();
});

loadExampleBtn.addEventListener("click", () => {
  vhdlEditor.value =
`library ieee;
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
  signal counter: unsigned(31 downto 0) := x"00000000";
begin
  led <= std_logic_vector(counter);
  process(clk)
  begin
    counter <= counter+1;
  end process;
end description;`;

  saveEditorToLocalStorageDebounced();
  updateLineNumbers();
});

// Editor events: save + line numbers + gutter sync
vhdlEditor.addEventListener("input", () => {
  saveEditorToLocalStorageDebounced();
  updateLineNumbers();
});

vhdlEditor.addEventListener("scroll", () => {
  syncGutterScroll();
});

// ----- Init -----
(function init() {
  wsUrlText.textContent = wsUrl();

  buildLeds();
  buildHex();
  buildSwitches();
  buildButtons();

  resetOutputsVisuals();
  setStatus(false);

  // Load from localStorage if present
  const saved = loadEditorFromLocalStorage();
  if (saved !== null) {
    vhdlEditor.value = saved;
  } else {
    vhdlEditor.value =
`-- circuit.vhdl
-- Paste your circuit here. The UI will send it on Connect.`;
  }

  updateLineNumbers();
})();