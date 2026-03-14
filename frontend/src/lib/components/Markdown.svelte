<script lang="ts">
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  interface Props {
    content: string;
  }
  let { content }: Props = $props();

  let html = $derived((() => {
    if (!content?.trim()) return '';
    const raw = marked.parse(content, { async: false }) as string;
    return DOMPurify.sanitize(raw);
  })());
</script>

{#if html}
  <div class="md-content">{@html html}</div>
{/if}
