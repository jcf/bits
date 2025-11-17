<script module>
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Hello from "./Hello.svelte";
  import { fn } from "storybook/test";

  const { Story } = defineMeta({
    title: "Example/Hello",
    component: Hello,
    args: {
      onclick: fn(),
    },
  });
</script>

<Story name="Primary" />
