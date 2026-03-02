<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/auth";

  onMount(() => {
    const unsubscribe = authStore.subscribe(state => {
      if (state.isAuthenticated) {
        goto("/depot");
      } else {
        goto("/login");
      }
    });
    return unsubscribe;
  });
</script>
