<script>
    import { Play, Pause, StepForward, Flag } from "lucide-svelte";
    import { onMount, onDestroy } from "svelte";

    export let buttons = [
        { icon: Play, tooltip: "Run (F5)", shortcut: "F5", action: () => console.log("Run") },
        { icon: Pause, tooltip: "Pause (F6)", shortcut: "F6", action: () => console.log("Pause") },
        { icon: StepForward, tooltip: "Step (F7)", shortcut: "F7", action: () => console.log("Step") },
        { icon: Flag, tooltip: "Toggle Breakpoint (F9)", shortcut: "F9", action: () => console.log("Toggle BP") },
    ];

    // Listen for keyboard shortcuts
    function handleKey(e) {
        const btn = buttons.find((b) => b.shortcut && e.key.toUpperCase() === b.shortcut.toUpperCase());
        if (btn) {
            e.preventDefault();
            btn.action();
        }
    }

    onMount(() => window.addEventListener("keydown", handleKey));
    onDestroy(() => window.removeEventListener("keydown", handleKey));
</script>

<div class="flex items-center gap-1 bg-gray-800 border-b border-gray-700 px-2 py-1 text-gray-200 shadow select-none">
    {#each buttons as btn}
        <button
            class="p-2 rounded hover:bg-gray-700 transition-colors"
            on:click={btn.action}
            title={btn.tooltip}
        >
            <svelte:component this={btn.icon} class="w-4 h-4" />
        </button>
    {/each}
</div>
