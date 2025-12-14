<script lang="ts">
  /**
   * QueryBuilder - Re-export from query-builder directory.
   * This file exists for backwards compatibility.
   */
  import { QueryBuilder } from "./query-builder";

  interface Props {
    onResultClick?: (noteId: number, notePath: string, noteTitle: string | null) => void;
  }

  let { onResultClick }: Props = $props();
</script>

<QueryBuilder {onResultClick} />
