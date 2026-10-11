import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const curlLanguage: CodeLanguage = {
  id: 'curl',
  name: 'cURL',
  icon: '⚡',
  variants: [
    { id: 'bash', name: 'cURL (Bash / zsh)' },
    { id: 'cmd', name: 'cURL (Windows CMD)' },
    { id: 'powershell', name: 'cURL (PowerShell)' }
  ],
  generate: (req, variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    if (variantId === 'cmd') {
      const lines = [`curl.exe -X ${method} "${url}"`];
      for (const [k, v] of Object.entries(headers)) {
        lines.push(`  -H "${k}: ${v}"`);
      }
      if (body && method !== 'GET' && method !== 'HEAD') {
        const escaped = body.replace(/"/g, '"""');
        lines.push(`  --data "${escaped}"`);
      }
      return lines.join(' ^\n');
    }

    if (variantId === 'powershell') {
      const lines = [`curl.exe -X ${method} "${url}"`];
      for (const [k, v] of Object.entries(headers)) {
        lines.push(`  -H "${k}: ${v}"`);
      }
      if (body && method !== 'GET' && method !== 'HEAD') {
        lines.push(`  --data '${body}'`);
      }
      return lines.join(' `\n');
    }

    // Default: bash
    const lines = [`curl -X ${method} "${url}"`];
    for (const [k, v] of Object.entries(headers)) {
      lines.push(`  -H "${k}: ${v}"`);
    }
    if (body && method !== 'GET' && method !== 'HEAD') {
      lines.push(`  -d '${body}'`);
    }
    return lines.join(' \\\n');
  }
};
