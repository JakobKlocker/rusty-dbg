<script context="module" lang="ts">
    export interface DisasmLine {
        address: number;
        bytes: number[];
        command: string;
    }
</script>

<script lang="ts">
    export let disassembly: DisasmLine[] = [];
    export let currentAddress: number = 0x0;

    function toHex(byte: number): string {
        return byte.toString(16).toUpperCase().padStart(2, "0");
    }

    function toAddr(addr: number): string {
        return "0x" + addr.toString(16).toUpperCase().padStart(4, "0");
    }
</script>

<div
    class="font-mono text-sm bg-gray-900 text-gray-300 p-2 rounded-lg shadow-lg overflow-y-auto"
>
    {#each disassembly as line}
        <div
            class="flex gap-4 py-0.5 px-1 {line.address === currentAddress
                ? 'bg-yellow-600 text-black'
                : 'hover:bg-gray-700'}"
        >
            <span class="text-green-400 w-20">{toAddr(line.address)}</span>

            <span class="text-purple-400 w-32">
                {#each line.bytes as b}
                    {toHex(b)}
                {/each}
            </span>

            <span class="text-blue-300 flex-1">{line.command}</span>
        </div>
    {/each}
</div>
