<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import DisassemblyWindow from "./components/DisassemblyWindow.svelte";
  import HexView from "./components/HexView.svelte";
  import Menu from "./components/Menu.svelte";
  import Toolbar from "./components/Toolbar.svelte";

  let isInitialized = false;
  let debugPath = "";
  let initError = "";

  // Example mock data
  const disasmData = [
    { address: 0x1000, bytes: [0x55], command: "push rbp" },
    { address: 0x1001, bytes: [0x48, 0x89, 0xE5], command: "mov rbp, rsp" },
    { address: 0x1004, bytes: [0x5D], command: "pop rbp" },
    { address: 0x1005, bytes: [0xC3], command: "ret" },
  ];

  let data: Uint8Array = new Uint8Array([
    0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x77,
    0x6f, 0x72, 0x6c, 0x64, 0x21, 0x00, 0x7f, 0xff,
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
  ]);

  let currentRIP = 0x1004;

  async function initDebugger() {
    try {
      await invoke("debugger_init", { path: debugPath });
      isInitialized = true;
    } catch (e) {
      console.error("Failed to initialize debugger:", e);
      initError = "Failed to initialize debugger!";
    }
  }
</script>

<!-- Fullscreen Init Screen -->
{#if !isInitialized}
  <div class="fixed inset-0 bg-gray-950 text-white flex items-center justify-center">
    <div class="bg-gray-800 p-6 rounded shadow-lg w-96">
      <h2 class="text-xl mb-4">Enter Path to Program</h2>
      <input
        type="text"
        class="w-full px-2 py-1 text-black rounded mb-3"
        bind:value={debugPath}
        placeholder="e.g., /path/to/program"
        on:keydown={(e) => e.key === "Enter" && initDebugger()}
      />
      <button
        class="w-full px-3 py-2 bg-blue-600 rounded hover:bg-blue-700"
        on:click={initDebugger}
      >
        Start Debugger
      </button>
      {#if initError}
        <p class="text-red-400 mt-2">{initError}</p>
      {/if}
    </div>
  </div>
{:else}
  <!-- Main App UI -->
  <main class="p-4 bg-gray-950 min-h-screen space-y-4">
    <Menu />
    <Toolbar />
    <DisassemblyWindow disassembly={disasmData} currentAddress={currentRIP} />
    <HexView data={data} />
  </main>
{/if}
