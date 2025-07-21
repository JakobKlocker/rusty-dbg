<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let message = '';
  let otherVal = '';

  async function callCustomCommand() {
    try {
      // Call Tauri command with argument
      const res = await invoke<{ message: string; other_val: string }>(
        'my_custom_command',
        { number: 42 }
      );

      // Extract response fields
      message = res.message;
      otherVal = res.other_val;
    } catch (e) {
      console.error('Error invoking command:', e);
      message = 'Command failed';
    }
  }
</script>

<main class="p-4">
  <button
    on:click={callCustomCommand}
    class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
  >
    Call Custom Command
  </button>

  {#if message}
    <p class="mt-4">Message: {message}</p>
    <p>Other Val: {otherVal}</p>
  {/if}
</main>
