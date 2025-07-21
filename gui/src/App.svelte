<script lang="ts">
import DisassemblyWindow from "./components/DisassemblyWindow.svelte";
import HexView from "./components/HexView.svelte";
import Menu from "./components/Menu.svelte";
import Toolbar from "./components/Toolbar.svelte";
  const disasmData = [
    { address: 0x1000, bytes: [0x55], command: 'push rbp' },
    { address: 0x1001, bytes: [0x48, 0x89, 0xE5], command: 'mov rbp, rsp' },
    { address: 0x1004, bytes: [0x5D], command: 'pop rbp' },
    { address: 0x1005, bytes: [0xC3], command: 'ret' },
  ];
   let data: Uint8Array = new Uint8Array([
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x77,
        0x6f, 0x72, 0x6c, 0x64, 0x21, 0x00, 0x7f, 0xff,
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
        0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48
    ]);


  let currentRIP = 0x1004;
</script>

<main class="p-4 bg-gray-950 min-h-screen space-y-4">
  <Menu/>
  <Toolbar/>
  <DisassemblyWindow disassembly ={disasmData} currentAddress={currentRIP} />
  <HexView data = {data}/>
</main>