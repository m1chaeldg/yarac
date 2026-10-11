import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const shellLanguage: CodeLanguage = {
  id: 'shell',
  name: 'Shell / CLI',
  icon: '🐚',
  variants: [
    { id: 'wget', name: 'GNU Wget' },
    { id: 'httpie', name: 'HTTPie' }
  ],
  generate: (req, variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    if (variantId === 'httpie') {
      const headerArgs = Object.entries(headers)
        .map(([k, v]) => `"${k}:${v}"`)
        .join(' \\\n  ');

      return `http ${method} "${url}" \\\n  ${headerArgs}${body ? ` \\\n  <<< '${body}'` : ''}`;
    }

    // Default: wget
    const lines = [`wget --method=${method} "${url}"`];
    for (const [k, v] of Object.entries(headers)) {
      lines.push(`  --header="${k}: ${v}"`);
    }
    if (body) {
      lines.push(`  --body-data='${body}'`);
    }
    lines.push('  -O -');
    return lines.join(' \\\n');
  }
};
