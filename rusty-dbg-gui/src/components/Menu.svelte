<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    let activeMenu: string | null = null;
    let debugPath = "";
    let initMessage = "";
    let showPopup = false;

    function openMenu(menu: string) {
        activeMenu = menu;
    }

    function closeMenus() {
        activeMenu = null;
    }

    function toggleMenu(menu: string) {
        activeMenu = activeMenu === menu ? null : menu;
    }

    function openPopup() {
        showPopup = true;
        activeMenu = null;
    }

    async function initDebugger() {
        if (!debugPath) return;
        try {
            await invoke("debugger_init", { path: debugPath });
            initMessage = `Debugger initialized with ${debugPath}`;
            showPopup = false;
        } catch (e) {
            console.error("Failed to init debugger:", e);
            initMessage = "Failed to initialize debugger!";
        }
    }

    onMount(() => {
        window.addEventListener("click", closeMenus);
    });
</script>

<div class="flex bg-gray-900 text-gray-200 text-sm px-2 py-1 shadow-md border-b border-gray-700 select-none">
    <!-- Open Menu -->
    <div 
        class="relative px-3 py-1 cursor-pointer hover:bg-gray-700"
        on:mouseenter={() => activeMenu && openMenu('open')}
        on:click={(e) => { e.stopPropagation(); toggleMenu('open'); }}
    >
        Open
        {#if activeMenu === 'open'}
            <div class="absolute left-0 top-full bg-gray-900 text-gray-200 rounded border border-gray-600 shadow-lg mt-1 w-44 z-50">
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer" on:click={openPopup}>
                    Open File...
                </div>
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Open Recent</div>
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Open Workspace</div>
            </div>
        {/if}
    </div>

    <!-- Breakpoints Menu -->
    <div 
        class="relative px-3 py-1 cursor-pointer hover:bg-gray-700"
        on:mouseenter={() => activeMenu && openMenu('breakpoints')}
        on:click={(e) => { e.stopPropagation(); toggleMenu('breakpoints'); }}
    >
        Breakpoints
        {#if activeMenu === 'breakpoints'}
            <div class="absolute left-0 top-full bg-gray-900 text-gray-200 rounded border border-gray-600 shadow-lg mt-1 w-52 z-50">
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Toggle Breakpoint</div>
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Remove All Breakpoints</div>
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Breakpoint Settings</div>
            </div>
        {/if}
    </div>
</div>

<!-- Popup Modal -->
{#if showPopup}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" on:click={() => (showPopup = false)}>
        <div class="bg-gray-900 text-white p-4 rounded shadow-lg w-80 relative" on:click|stopPropagation>
            <h2 class="text-lg mb-2">Enter Debug Path</h2>
            <input 
                class="w-full px-2 py-1 text-black rounded mb-3"
                type="text"
                bind:value={debugPath}
                placeholder="e.g., /path/to/program"
                on:keydown={(e) => e.key === 'Enter' && initDebugger()}
            />
            <div class="flex justify-end gap-2">
                <button class="px-3 py-1 bg-gray-600 rounded hover:bg-gray-700" on:click={() => (showPopup = false)}>Cancel</button>
                <button class="px-3 py-1 bg-blue-600 rounded hover:bg-blue-700" on:click={initDebugger}>Initialize</button>
            </div>
        </div>
    </div>
{/if}

{#if initMessage}
    <p class="p-2 text-sm text-green-400">{initMessage}</p>
{/if}
