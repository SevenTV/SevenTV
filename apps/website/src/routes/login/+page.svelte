<script lang="ts">
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
</script>

<SignInDialog mode={DialogMode.ShownWithoutClose} />
