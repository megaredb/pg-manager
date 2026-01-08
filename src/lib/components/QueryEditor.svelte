<script lang="ts">
  import CodeMirror from "svelte-codemirror-editor";
  import { sql } from "@codemirror/lang-sql";
  import { oneDark } from "@codemirror/theme-one-dark";

  interface Props {
    value: string;
    onChange: (val: string) => void;
    onRun: () => void;
  }

  let { value = $bindable(), onChange, onRun }: Props = $props();

  function handleChange() {
    if (onChange) {
      onChange(value);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="h-full w-full text-base overflow-scroll"
  role="group"
  onkeydown={(e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      onRun();
    }
  }}
>
  <CodeMirror
    bind:value
    lang={sql()}
    theme={oneDark}
    styles={{
      "&": { height: "100%", width: "100%" },
    }}
    onchange={handleChange}
  />
</div>
