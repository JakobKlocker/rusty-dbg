<script lang="ts">
    import { onMount } from "svelte";

    export let data: Uint8Array = new Uint8Array();
    export let baseAddress = 0x1000;
    export let bytesPerRow = 16;

    let contextMenuVisible = false;
    let contextMenuX = 0;
    let contextMenuY = 0;
    let selectedByteIndex: number | null = null;
    let contextType: "byte" | "view" = "view";

    function toHex(byte: number): string {
        return byte.toString(16).toUpperCase().padStart(2, "0");
    }

    function toAscii(byte: number): string {
        return byte >= 32 && byte < 127 ? String.fromCharCode(byte) : ".";
    }

    function openContextMenu(
        event: MouseEvent,
        type: "byte" | "view",
        index: number | null = null,
    ) {
        event.preventDefault();
        event.stopPropagation();
        contextMenuX = event.clientX;
        contextMenuY = event.clientY;
        contextMenuVisible = true;
        selectedByteIndex = index;
        contextType = type;
    }

    function goToAddress() {
        const newAddr = prompt(
            "Enter new base address (hex):",
            baseAddress.toString(16).toUpperCase(),
        );
        if (newAddr && /^[0-9A-Fa-f]+$/.test(newAddr)) {
            baseAddress = parseInt(newAddr, 16);
        }
        contextMenuVisible = false;
    }

    function changeByte() {
        if (selectedByteIndex !== null) {
            const currentValue = toHex(data[selectedByteIndex]);
            const newVal = prompt(
                `Enter new hex value (00-FF) for byte at index ${selectedByteIndex}:`,
                currentValue,
            );
            if (newVal && /^[0-9A-Fa-f]{1,2}$/.test(newVal)) {
                data[selectedByteIndex] = parseInt(newVal, 16);
            }
        }
        contextMenuVisible = false;
    }

    function closeContextMenu() {
        contextMenuVisible = false;
    }

    onMount(() => {
        window.addEventListener("click", closeContextMenu);
    });
</script>

<div
    role="region"
    class="font-mono text-sm bg-gray-900 text-gray-300 p-2 overflow-x-auto rounded-lg shadow-lg relative"
    on:contextmenu={(e) => openContextMenu(e, "view")}
>
    {#each Array(Math.ceil(data.length / bytesPerRow)) as _, rowIndex}
        <div class="flex gap-4 whitespace-pre">
            <span class="text-green-400 w-20 text-right">
                {(baseAddress + rowIndex * bytesPerRow)
                    .toString(16)
                    .toUpperCase()
                    .padStart(8, "0")}:
            </span>

            <span class="text-purple-400 min-w-[240px]">
                {#each data.slice(rowIndex * bytesPerRow, (rowIndex + 1) * bytesPerRow) as byte, i}
                    <span
                        role="button"
                        tabindex="0"
                        class="px-1 hover:bg-gray-700 cursor-pointer"
                        on:contextmenu={(e) =>
                            openContextMenu(
                                e,
                                "byte",
                                rowIndex * bytesPerRow + i,
                            )}
                    >
                        {toHex(byte)}
                    </span>
                {/each}
            </span>

            <span class="text-yellow-300">
                {#each data.slice(rowIndex * bytesPerRow, (rowIndex + 1) * bytesPerRow) as byte}
                    {toAscii(byte)}
                {/each}
            </span>
        </div>
    {/each}

    {#if contextMenuVisible}
        <div
            class="fixed bg-gray-800 text-white text-sm rounded shadow-lg py-1 z-50"
            style="top:{contextMenuY}px; left:{contextMenuX}px;"
        >
            {#if contextType === "byte"}
                <button
                    class="px-3 py-1 hover:bg-gray-600 cursor-pointer w-full text-left"
                    on:click={changeByte}
                >
                    Change
                </button>
            {:else}
                <button
                    class="px-3 py-1 hover:bg-gray-600 cursor-pointer w-full text-left"
                    on:click={goToAddress}
                >
                    Go To Address
                </button>
            {/if}
        </div>
    {/if}
</div>

<style>
    span {
        user-select: none;
    }
</style>
