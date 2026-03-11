const LS_KEY_VHDL = "circuit_ui:circuit.vhdl";
const LS_KEY_MODE = "circuit_ui:mode";

const EXAMPLE_VHDL_TEXT = `library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Do not modify the following entity block
entity circuit is
port (
  clk: in std_logic; -- 500 Hz, period 2 ms
  btn: in std_logic_vector(31 downto 0);
  sw: in std_logic_vector(31 downto 0);
  led: out std_logic_vector(31 downto 0) := (others => '0');
  seg0: out std_logic_vector(31 downto 0);
  seg1: out std_logic_vector(31 downto 0);
  seg2: out std_logic_vector(31 downto 0);
  seg3: out std_logic_vector(31 downto 0)
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

function getDomRefs() {
  return {
    statusPill: document.getElementById("statusPill"),
    modeToggle: document.getElementById("modeToggle"),
    connectToggleBtn: document.getElementById("connectToggleBtn"),
    runToggleBtn: document.getElementById("runToggleBtn"),

    editorSection: document.getElementById("editorSection"),
    vhdlEditor: document.getElementById("vhdlEditor"),
    lineGutter: document.getElementById("lineGutter"),
    loadExampleBtn: document.getElementById("loadExampleBtn"),

    ledRow: document.getElementById("ledRow"),
    hexRow: document.getElementById("hexRow"),

    switchGrid: document.getElementById("switchGrid"),
    buttonGrid: document.getElementById("buttonGrid"),
    keypadGrid: document.getElementById("keypadGrid"),
    allSwOffBtn: document.getElementById("allSwOffBtn"),
    allSwOnBtn: document.getElementById("allSwOnBtn"),

    logView: document.getElementById("logView"),
    clearLogBtn: document.getElementById("clearLogBtn"),
  };
}

function parseBoolean(value) {
  if (typeof value === "boolean") return value;
  if (typeof value !== "string") return false;
  const normalized = value.trim().toLowerCase();
  return normalized === "1" || normalized === "true" || normalized === "yes" || normalized === "on";
}

function getWebSocketUrl() {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  return `${proto}://${location.host}`;
}

function u32BitGet(value, bitIndex) {
  return ((value >>> bitIndex) & 1) === 1;
}

function u32BitSet(value, bitIndex, enabled) {
  if (enabled) return (value | (1 << bitIndex)) >>> 0;
  return (value & ~(1 << bitIndex)) >>> 0;
}

function parseHexDigits(hexU32) {
  return [
    (hexU32 >>> 0) & 0xff,
    (hexU32 >>> 8) & 0xff,
    (hexU32 >>> 16) & 0xff,
    (hexU32 >>> 24) & 0xff,
  ];
}

function parseSegRowBytes(rawValue) {
  // Accept [b0..b7], bigint, number, or numeric string (decimal / 0x-prefixed).
  if (Array.isArray(rawValue)) {
    const out = new Uint8Array(8);
    for (let i = 0; i < 8 && i < rawValue.length; i += 1) {
      out[i] = Number(rawValue[i]) & 0xff;
    }
    return out;
  }

  let valueBigInt = null;

  if (typeof rawValue === "bigint") {
    valueBigInt = rawValue;
  } else if (typeof rawValue === "number" && Number.isFinite(rawValue)) {
    valueBigInt = BigInt(Math.trunc(rawValue));
  } else if (typeof rawValue === "string") {
    const text = rawValue.trim();
    if (text.length === 0) return null;
    try {
      valueBigInt = BigInt(text);
    } catch {
      return null;
    }
  } else {
    return null;
  }

  const out = new Uint8Array(8);
  for (let i = 0; i < 8; i += 1) {
    out[i] = Number((valueBigInt >> BigInt(i * 8)) & 0xffn);
  }
  return out;
}

function isUnitMessage(msg, name) {
  if (msg === name) return true;
  if (msg && typeof msg === "object" && msg[name] !== undefined) return true;
  return false;
}

class LogController {
  constructor({ logView, clearLogBtn }) {
    this.logView = logView;
    this.clearLogBtn = clearLogBtn;
  }

  init() {
    this.clearLogBtn.addEventListener("click", () => this.clear());
  }

  append(stream, line) {
    const prefix = stream === "stderr" ? "[stderr]" : "[stdout]";
    this.logView.textContent += `${prefix} ${line}\n`;
    this.logView.scrollTop = this.logView.scrollHeight;
  }

  clear() {
    this.logView.textContent = "";
  }
}

class EditorController {
  constructor({ editorSection, vhdlEditor, lineGutter, loadExampleBtn, enabled, externalFiles }) {
    this.editorSection = editorSection;
    this.vhdlEditor = vhdlEditor;
    this.lineGutter = lineGutter;
    this.loadExampleBtn = loadExampleBtn;

    this.enabled = Boolean(enabled);
    this.externalFiles = externalFiles && typeof externalFiles === "object" ? externalFiles : null;
    this.saveTimer = null;
    this.initialized = false;
  }

  init() {
    this.setEnabled(this.enabled);
  }

  setEnabled(enabled) {
    this.enabled = Boolean(enabled);

    if (!this.enabled) {
      this.editorSection.classList.add("is-hidden");
      return;
    }

    this.initializeIfNeeded();
    this.editorSection.classList.remove("is-hidden");
    this.updateLineNumbers();
  }

  initializeIfNeeded() {
    if (this.initialized) return;
    this.initialized = true;

    const saved = this.loadFromLocalStorage();
    this.vhdlEditor.value = saved !== null ? saved : EXAMPLE_VHDL_TEXT;

    this.loadExampleBtn.addEventListener("click", () => {
      this.vhdlEditor.value = EXAMPLE_VHDL_TEXT;
      this.saveToLocalStorageDebounced();
      this.updateLineNumbers();
    });

    this.vhdlEditor.addEventListener("input", () => {
      this.saveToLocalStorageDebounced();
      this.updateLineNumbers();
    });

    this.vhdlEditor.addEventListener("scroll", () => {
      this.lineGutter.scrollTop = this.vhdlEditor.scrollTop;
    });
  }

  getFilesPayload() {
    if (!this.enabled) {
      return this.externalFiles ? { ...this.externalFiles } : {};
    }

    return {
      "circuit.vhdl": this.vhdlEditor.value ?? "",
    };
  }

  updateLineNumbers() {
    const text = this.vhdlEditor.value || "";
    const lineCount = text.length ? text.split("\n").length : 1;

    let gutterText = "";
    for (let i = 1; i <= lineCount; i += 1) {
      gutterText += `${i}\n`;
    }

    this.lineGutter.textContent = gutterText;
  }

  saveToLocalStorageDebounced() {
    if (this.saveTimer) clearTimeout(this.saveTimer);

    this.saveTimer = setTimeout(() => {
      try {
        localStorage.setItem(LS_KEY_VHDL, this.vhdlEditor.value ?? "");
      } catch {
        // Ignore localStorage failures.
      }
    }, 250);
  }

  loadFromLocalStorage() {
    try {
      const saved = localStorage.getItem(LS_KEY_VHDL);
      if (saved !== null) return saved;
    } catch {
      // Ignore localStorage failures.
    }
    return null;
  }
}

class OutputController {
  constructor({ ledRow, hexRow }) {
    this.ledRow = ledRow;
    this.hexRow = hexRow;

    this.ledEls = Array.from(this.ledRow.querySelectorAll(".led[data-bit]"));
    this.segDisplays = Array.from(this.hexRow.querySelectorAll(".sevenSeg[data-digit]"));

    this.segBytes = new Uint8Array(this.segDisplays.length);
    this.segMap = { a: 0, b: 1, c: 2, d: 3, e: 4, f: 5, g: 6, dp: 7 };
  }

  init() {
    this.resetVisuals();
  }

  resetVisuals() {
    this.setLeds(0);
    this.segBytes.fill(0);
    this.renderAllSegments();
  }

  handleMessage(parsed) {
    if (parsed.led !== undefined) {
      const value = (parsed.led ?? parsed.value ?? parsed[0] ?? parsed["0"] ?? 0) >>> 0;
      this.setLeds(value);
      return true;
    }

    let handledSegment = false;

    // Row mapping:
    // seg0 -> displays 0..7
    // seg1 -> displays 8..15
    // seg2 -> displays 16..23
    // seg3 -> displays 24..31
    for (let row = 0; row < 4; row += 1) {
      const key = `seg${row}`;
      if (parsed[key] === undefined) continue;

      const rowBytes = parseSegRowBytes(parsed[key]);
      if (!rowBytes) continue;

      this.setSegRow(row, rowBytes);
      handledSegment = true;
    }

    // Backward-compat path for a single 32-bit value (fills first 4 displays).
    if (!handledSegment && parsed.seg !== undefined) {
      const bytes = parseHexDigits(Number(parsed.seg) >>> 0);
      this.setSegRow(0, bytes);
      handledSegment = true;
    }

    if (handledSegment) {
      this.renderAllSegments();
      return true;
    }

    return false;
  }

  setLeds(bitsU32) {
    for (const led of this.ledEls) {
      const bit = Number(led.dataset.bit);
      led.classList.toggle("on", u32BitGet(bitsU32, bit));
    }
  }

  renderAllSegments() {
    for (let i = 0; i < this.segDisplays.length; i += 1) {
      const display = this.segDisplays[i];
      const byte = this.segBytes[i] & 0xff;

      for (const segEl of display.querySelectorAll(".seg")) {
        const segName = segEl.dataset.seg;
        const bit = this.segMap[segName];
        const on = ((byte >>> bit) & 1) === 1;
        segEl.classList.toggle("on", on);
      }
    }
  }

  setSegRow(rowIndex, bytes) {
    const base = rowIndex * 8;
    for (let i = 0; i < 8; i += 1) {
      const dst = base + i;
      if (dst >= this.segBytes.length) break;
      this.segBytes[dst] = bytes[i] & 0xff;
    }
  }
}

class InputController {
  constructor({ switchGrid, buttonGrid, keypadGrid, allSwOffBtn, allSwOnBtn, sendClientInput }) {
    this.switchGrid = switchGrid;
    this.buttonGrid = buttonGrid;
    this.keypadGrid = keypadGrid;
    this.allSwOffBtn = allSwOffBtn;
    this.allSwOnBtn = allSwOnBtn;
    this.sendClientInput = sendClientInput;

    this.switches = 0 >>> 0;
    this.btn = 0 >>> 0;
    this.matrixPressCounts = new Map();
  }

  init() {
    this.bindSwitches();
    this.bindStandardButtons();
    this.bindKeypadMatrix();

    this.allSwOffBtn.addEventListener("click", () => {
      this.switches = 0 >>> 0;
      this.syncSwitchesUI();
      this.publishInput();
    });

    this.allSwOnBtn.addEventListener("click", () => {
      this.switches = 0xffff_ffff >>> 0;
      this.syncSwitchesUI();
      this.publishInput();
    });
  }

  getInputPayload() {
    return {
      switch: this.switches >>> 0,
      buttons: this.btn >>> 0,
    };
  }

  publishInput() {
    this.sendClientInput(this.getInputPayload());
  }

  bindSwitches() {
    const toggles = this.switchGrid.querySelectorAll(".toggle[data-bit]");

    for (const toggle of toggles) {
      toggle.addEventListener("click", () => {
        const bit = Number(toggle.dataset.bit);
        const nextValue = !u32BitGet(this.switches, bit);
        this.switches = u32BitSet(this.switches, bit, nextValue);
        toggle.classList.toggle("on", nextValue);
        this.publishInput();
      });
    }
  }

  syncSwitchesUI() {
    const toggles = this.switchGrid.querySelectorAll(".toggle[data-bit]");

    for (const toggle of toggles) {
      const bit = Number(toggle.dataset.bit);
      toggle.classList.toggle("on", u32BitGet(this.switches, bit));
    }
  }

  bindStandardButtons() {
    const buttons = this.buttonGrid.querySelectorAll(".momentary[data-bit]");

    for (const button of buttons) {
      let isPressed = false;

      const press = () => {
        if (isPressed) return;
        isPressed = true;
        const bit = Number(button.dataset.bit);
        this.btn = u32BitSet(this.btn, bit, true);
        button.classList.add("down");
        this.publishInput();
      };

      const release = () => {
        if (!isPressed) return;
        isPressed = false;
        const bit = Number(button.dataset.bit);
        this.btn = u32BitSet(this.btn, bit, false);
        button.classList.remove("down");
        this.publishInput();
      };

      button.addEventListener("mousedown", (event) => {
        event.preventDefault();
        press();
      });
      button.addEventListener("mouseup", (event) => {
        event.preventDefault();
        release();
      });
      button.addEventListener("mouseleave", (event) => {
        event.preventDefault();
        release();
      });

      button.addEventListener(
        "touchstart",
        (event) => {
          event.preventDefault();
          press();
        },
        { passive: false },
      );
      button.addEventListener(
        "touchend",
        (event) => {
          event.preventDefault();
          release();
        },
        { passive: false },
      );
      button.addEventListener(
        "touchcancel",
        (event) => {
          event.preventDefault();
          release();
        },
        { passive: false },
      );
    }
  }

  bumpMatrixBit(bit, delta) {
    const current = this.matrixPressCounts.get(bit) ?? 0;
    const next = Math.max(0, current + delta);
    this.matrixPressCounts.set(bit, next);

    const active = next > 0;
    this.btn = u32BitSet(this.btn, bit, active);
  }

  bindKeypadMatrix() {
    const keys = this.keypadGrid.querySelectorAll(".keypadBtn[data-row-bit][data-col-bit]");

    for (const keyButton of keys) {
      const rowBit = Number(keyButton.dataset.rowBit);
      const colBit = Number(keyButton.dataset.colBit);
      let isPressed = false;

      const press = () => {
        if (isPressed) return;
        isPressed = true;
        this.bumpMatrixBit(rowBit, +1);
        this.bumpMatrixBit(colBit, +1);
        keyButton.classList.add("down");
        this.publishInput();
      };

      const release = () => {
        if (!isPressed) return;
        isPressed = false;
        this.bumpMatrixBit(rowBit, -1);
        this.bumpMatrixBit(colBit, -1);
        keyButton.classList.remove("down");
        this.publishInput();
      };

      keyButton.addEventListener("mousedown", (event) => {
        event.preventDefault();
        press();
      });
      keyButton.addEventListener("mouseup", (event) => {
        event.preventDefault();
        release();
      });
      keyButton.addEventListener("mouseleave", (event) => {
        event.preventDefault();
        release();
      });

      keyButton.addEventListener(
        "touchstart",
        (event) => {
          event.preventDefault();
          press();
        },
        { passive: false },
      );
      keyButton.addEventListener(
        "touchend",
        (event) => {
          event.preventDefault();
          release();
        },
        { passive: false },
      );
      keyButton.addEventListener(
        "touchcancel",
        (event) => {
          event.preventDefault();
          release();
        },
        { passive: false },
      );
    }
  }
}

class ConnectionController {
  constructor({ connectToggleBtn, logger, onOpen, onMessage, onClose, onBeforeConnect, wsUrlFactory }) {
    this.connectToggleBtn = connectToggleBtn;
    this.logger = logger;
    this.onOpen = onOpen;
    this.onMessage = onMessage;
    this.onClose = onClose;
    this.onBeforeConnect = onBeforeConnect;
    this.wsUrlFactory = wsUrlFactory;

    this.ws = null;
    this.connected = false;
  }

  init() {
    this.setStatus(false);
    this.connectToggleBtn.addEventListener("click", () => this.toggleConnect());
  }

  isConnected() {
    return this.connected;
  }

  setStatus(connected) {
    this.connected = connected;

    this.connectToggleBtn.textContent = connected ? "Disconnect" : "Connect";
    this.connectToggleBtn.classList.toggle("secondary", connected);
  }

  send(data) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    this.ws.send(JSON.stringify(data));
  }

  toggleConnect() {
    if (this.isConnected()) {
      this.disconnect();
      return;
    }

    this.connect();
  }

  connect() {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.onBeforeConnect();

    const url = this.wsUrlFactory();
    this.ws = new WebSocket(url);

    this.ws.addEventListener("open", () => {
      this.setStatus(true);
      this.logger.append("stdout", "WebSocket connected.");
      this.onOpen();
    });

    this.ws.addEventListener("message", (event) => {
      let parsed = null;
      try {
        parsed = JSON.parse(event.data);
      } catch {
        this.logger.append("stderr", String(event.data));
        return;
      }

      this.onMessage(parsed, event.data);
    });

    this.ws.addEventListener("close", () => {
      this.logger.append("stderr", "WebSocket closed.");
      this.setStatus(false);
      this.onClose();
    });

    this.ws.addEventListener("error", () => {
      this.logger.append("stderr", "WebSocket error.");
      this.setStatus(false);
    });
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }
}

class CircuitUiApp {
  constructor(config) {
    this.config = config;
    this.dom = getDomRefs();
    this.mode = config.initialMode;
    this.isRunning = false;
    this.reconnectTimer = null;

    this.logs = new LogController(this.dom);

    this.editor = new EditorController({
      ...this.dom,
      enabled: config.initialMode === "uploaded",
      externalFiles: config.externalFiles,
    });

    this.outputs = new OutputController(this.dom);

    this.inputs = new InputController({
      ...this.dom,
      sendClientInput: (payload) => {
        this.connection.send({ input: payload });
      },
    });

    this.connection = new ConnectionController({
      connectToggleBtn: this.dom.connectToggleBtn,
      logger: this.logs,
      wsUrlFactory: () => `${getWebSocketUrl()}/ws/${this.mode}`,
      onBeforeConnect: () => {
        this.logs.clear();
        this.outputs.resetVisuals();
      },
      onOpen: () => {
        if (this.mode === "uploaded") {
          this.connection.send(this.editor.getFilesPayload());
        }
        this.connection.send({ input: this.inputs.getInputPayload() });
        this.setRunButtonEnabled(true);
        this.updateStatusIndicator();
      },
      onMessage: (parsed, raw) => {
        if (isUnitMessage(parsed, "start")) {
          this.logs.clear();
          this.outputs.resetVisuals();
          this.setRunning(true);
          return;
        }

        if (isUnitMessage(parsed, "stop")) {
          this.setRunning(false);
          return;
        }

        if (parsed.log !== undefined) {
          this.logs.append(parsed.log.stream ?? "stdout", parsed.log.line ?? "");
          return;
        }

        if (this.outputs.handleMessage(parsed)) {
          return;
        }

        this.logs.append("stderr", `Unknown msg: ${raw}`);
      },
      onClose: () => {
        this.setRunButtonEnabled(false);
        this.setRunning(false);
        this.updateStatusIndicator();
        if (this.mode === "workspace") {
          this.scheduleReconnect();
        }
      },
    });
  }

  init() {
    this.logs.init();
    this.editor.init();
    this.outputs.init();
    this.inputs.init();
    this.connection.init();
    this.wireModeControls();
    this.wireRunControls();
    this.applyMode(this.mode, true);
  }

  wireRunControls() {
    this.setRunning(false);
    this.setRunButtonEnabled(false);
    this.updateStatusIndicator();

    this.dom.runToggleBtn.addEventListener("click", () => {
      if (!this.connection.isConnected()) return;

      if (this.isRunning) {
        this.connection.send({ stop: null });
      } else {
        this.connection.send({ start: null });
      }
    });
  }

  wireModeControls() {
    this.dom.modeToggle.addEventListener("change", () => {
      const nextMode = this.dom.modeToggle.checked ? "uploaded" : "workspace";
      this.applyMode(nextMode);
    });
  }

  applyMode(nextMode, fromInit = false) {
    const mode =
      nextMode === "workspace" && this.config.workspaceEnabled ? "workspace" : "uploaded";
    const changed = this.mode !== mode;

    if (!fromInit && changed && this.connection.isConnected()) {
      this.connection.disconnect();
    }

    this.mode = mode;
    try {
      localStorage.setItem(LS_KEY_MODE, mode);
    } catch {}

    const isUploaded = mode === "uploaded";
    this.dom.modeToggle.checked = isUploaded;
    this.dom.modeToggle.disabled = !this.config.workspaceEnabled;
    this.editor.setEnabled(isUploaded);
    this.dom.connectToggleBtn.classList.toggle("is-hidden", !isUploaded);
    this.dom.runToggleBtn.classList.toggle("is-hidden", isUploaded);

    if (!isUploaded) {
      this.scheduleReconnect(0);
    } else {
      this.cancelReconnect();
    }

    this.updateStatusIndicator();
  }

  setRunning(running) {
    this.isRunning = Boolean(running);
    this.dom.runToggleBtn.textContent = this.isRunning ? "Stop" : "Start";
    this.dom.runToggleBtn.classList.toggle("secondary", this.isRunning);
    this.updateStatusIndicator();
  }

  setRunButtonEnabled(enabled) {
    this.dom.runToggleBtn.disabled = !enabled;
    this.updateStatusIndicator();
  }

  scheduleReconnect(delayMs = 800) {
    if (this.reconnectTimer !== null) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (!this.connection.isConnected()) {
        this.connection.connect();
      }
    }, delayMs);
  }

  cancelReconnect() {
    if (this.reconnectTimer === null) return;
    clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;
  }

  updateStatusIndicator() {
    const pill = this.dom.statusPill;
    const connected = this.connection.isConnected();
    const running = connected && this.isRunning;

    pill.classList.remove("state-disabled", "state-connected", "state-running");

    if (!connected) {
      pill.textContent = "DISABLED";
      pill.classList.add("state-disabled");
      return;
    }

    if (running) {
      pill.textContent = "RUNNING";
      pill.classList.add("state-running");
      return;
    }

    pill.textContent = "CONNECTED";
    pill.classList.add("state-connected");
  }
}

function resolveConfig() {
  const config = window.VHDL_UI_CONFIG ?? {};
  const query = new URLSearchParams(location.search);
  const queryMode = (query.get("mode") ?? "").toLowerCase();
  const workspaceEnabled = config.workspaceEnabled !== false;

  let storedMode = "";
  try {
    storedMode = (localStorage.getItem(LS_KEY_MODE) ?? "").toLowerCase();
  } catch {}

  let initialMode = workspaceEnabled ? "workspace" : "uploaded";
  if (queryMode === "workspace" || queryMode === "uploaded") {
    initialMode = queryMode;
  } else if (storedMode === "workspace" || storedMode === "uploaded") {
    initialMode = storedMode;
  } else if (config.mode === "workspace" || config.mode === "uploaded") {
    initialMode = config.mode;
  } else if (query.has("externalEditor")) {
    initialMode = parseBoolean(query.get("externalEditor")) ? "uploaded" : "workspace";
  } else if (parseBoolean(config.externalEditor)) {
    initialMode = "uploaded";
  }

  return {
    initialMode: initialMode === "workspace" && !workspaceEnabled ? "uploaded" : initialMode,
    workspaceEnabled,
    externalFiles: config.externalFiles ?? null,
  };
}

(function bootstrap() {
  const app = new CircuitUiApp(resolveConfig());
  app.init();
})();
