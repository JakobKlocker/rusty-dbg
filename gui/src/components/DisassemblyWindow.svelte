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

    let contextMenuVisible = false;
    let contextMenuX = 0;
    let contextMenuY = 0;

    let editRowIndex: number | null = null;
    let editedBytesString: string = "";

    function openRowEditor(lineIndex: number) {
        editRowIndex = lineIndex;
        editedBytesString = disassembly[lineIndex].bytes.map(toHex).join(" ");
    }

    function saveRow() {
        if (editRowIndex === null) return;

        const bytes = editedBytesString
            .trim()
            .split(/\s+/)
            .filter((b) => /^[0-9A-Fa-f]{2}$/.test(b))
            .map((b) => parseInt(b, 16));

        disassembly[editRowIndex].bytes = bytes;
        editRowIndex = null;
    }

    function cancelEdit() {
        editRowIndex = null;
    }

    function toHex(byte: number): string {
        return byte.toString(16).toUpperCase().padStart(2, "0");
    }

    function toAddr(addr: number): string {
        return "0x" + addr.toString(16).toUpperCase().padStart(4, "0");
    }

    function openContextMenu(event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();
        contextMenuX = event.clientX;
        contextMenuY = event.clientY;
        contextMenuVisible = true;
    }

    function goToAddress() {
        const newAddr = prompt(
            "Enter new base address (hex):",
            currentAddress.toString(16).toUpperCase(),
        );
        if (newAddr && /^[0-9A-Fa-f]+$/.test(newAddr)) {
            currentAddress = parseInt(newAddr, 16);
        }
        contextMenuVisible = false;
    }

    function closeContextMenu() {
        contextMenuVisible = false;
    }

    import { onMount } from "svelte";
    onMount(() => {
        window.addEventListener("click", closeContextMenu);
        window.addEventListener("keydown", (e) => {
            if (editRowIndex !== null) {
                if (e.key === "Escape") cancelEdit();
                if (e.key === "Enter") saveRow();
            }
        });
    });
</script>

<div
    role="region"
    class="font-mono text-sm bg-gray-900 text-gray-300 p-2 rounded-lg shadow-lg overflow-y-auto"
    on:contextmenu={openContextMenu}
>
    {#each disassembly as line, lineIndex}
        <div
            role="region"
            class="flex gap-4 py-0.5 px-1 {line.address === currentAddress
                ? 'bg-yellow-600 text-black'
                : 'hover:bg-gray-700'}"
            on:dblclick={() => openRowEditor(lineIndex)}
        >
            <span class="text-green-400 w-20">{toAddr(line.address)}</span>
            <span class="text-purple-400 flex-1">
                {#each line.bytes as b}
                    {toHex(b)}
                {/each}
            </span>
            <span class="text-blue-300">{line.command}</span>
        </div>
    {/each}

    {#if contextMenuVisible}
        <div
            class="fixed bg-gray-800 text-white text-sm rounded shadow-lg py-1 z-50"
            style="top:{contextMenuY}px; left:{contextMenuX}px;"
        >
            <button
                class="px-3 py-1 hover:bg-gray-600 cursor-pointer w-full text-left"
                on:click={goToAddress}
            >
                Go To Address
            </button>
        </div>
    {/if}

    {#if editRowIndex !== null}
        <!-- Popup Editor -->
        <div
            class="fixed inset-0 bg-white/10 flex items-center justify-center z-50"
        >
            <div class="bg-gray-900 text-white p-4 rounded-lg shadow-lg w-96">
                <h2 class="text-lg mb-2">Edit Bytes</h2>
                <input
                    class="w-full bg-gray-800 text-purple-300 px-2 py-1 rounded font-mono"
                    bind:value={editedBytesString}
                />
                <div class="mt-3 flex justify-end gap-2">
                    <button
                        class="px-3 py-1 bg-green-600 rounded"
                        on:click={saveRow}>Save</button
                    >
                    <button
                        class="px-3 py-1 bg-red-600 rounded"
                        on:click={cancelEdit}>Cancel</button
                    >
                </div>
                <p class="text-xs text-gray-400 mt-2">
                    Press Enter to Save, Escape to Cancel
                </p>
            </div>
        </div>
    {/if}
</div>

<style>
    span {
        user-select: none;
    }
    input {
        font-family: monospace;
    }
</style>
