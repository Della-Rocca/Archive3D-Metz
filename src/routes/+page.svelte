<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth.svelte";

  $effect(() => {
    if (auth.isAuthenticated) {
      goto("/depot");
    } else {
      goto("/login");
    }
  });
</script>
