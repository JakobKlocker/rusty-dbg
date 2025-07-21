<script>
    import { onMount } from "svelte";

    let activeMenu = null;

    function openMenu(menu) {
        activeMenu = menu;
    }

    function closeMenus() {
        activeMenu = null;
    }

    function toggleMenu(menu) {
        activeMenu = activeMenu === menu ? null : menu;
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
                <div class="px-3 py-1 hover:bg-gray-700 cursor-pointer">Open File...</div>
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

    <!-- Add more menus as needed -->
</div>
