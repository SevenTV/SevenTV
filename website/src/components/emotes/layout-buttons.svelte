<script lang="ts">
	import { Layout, emotesLayout } from "$/store/layout";
	import { GridFour, GridNine } from "phosphor-svelte";
	import Button from "../input/button.svelte";
</script>

<Button
	secondary={$emotesLayout === Layout.BigGrid}
	on:click={() => ($emotesLayout = Layout.BigGrid)}
>
	<GridFour slot="icon" />
</Button>
<Button
	secondary={$emotesLayout === Layout.SmallGrid}
	on:click={() => ($emotesLayout = Layout.SmallGrid)}
>
	<GridNine slot="icon" />
</Button>
