import type { YaracRequest } from '../../stores/workspace.svelte';

export interface CodeLanguage {
  id: string;
  name: string;
  icon: string;
  variants: Array<{ id: string; name: string }>;
  generate: (req: YaracRequest, variantId: string, envVars?: Record<string, string>) => string;
}
