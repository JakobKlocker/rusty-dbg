<script lang="ts">
import DisassemblyWindow from "./components/DisassemblyWindow.svelte";
  const disasmData = [
    { address: 0x1000, bytes: [0x55], command: 'push rbp' },
    { address: 0x1001, bytes: [0x48, 0x89, 0xE5], command: 'mov rbp, rsp' },
    { address: 0x1004, bytes: [0x5D], command: 'pop rbp' },
    { address: 0x1005, bytes: [0xC3], command: 'ret' },
  ];

  let currentRIP = 0x1004;
</script>

<main class="p-4 bg-gray-950 min-h-screen space-y-4">
  <h1 class="text-2xl text-white">Disassembly</h1>
  <DisassemblyWindow disassembly ={disasmData} currentAddress={currentRIP} />
</main>