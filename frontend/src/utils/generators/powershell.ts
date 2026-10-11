import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const powershellLanguage: CodeLanguage = {
  id: 'powershell',
  name: 'PowerShell',
  icon: '🟦',
  variants: [
    { id: 'restmethod', name: 'Invoke-RestMethod' },
    { id: 'webrequest', name: 'Invoke-WebRequest' }
  ],
  generate: (req, variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    const headerLines = Object.entries(headers)
      .map(([k, v]) => `    "${k}" = "${v}"`)
      .join('\n');

    const headerBlock = Object.keys(headers).length > 0
      ? `$headers = @{\n${headerLines}\n}\n`
      : '';

    const bodyBlock = body
      ? `$body = @'\n${body}\n'@\n`
      : '';

    const cmd = variantId === 'webrequest' ? 'Invoke-WebRequest' : 'Invoke-RestMethod';

    return `${headerBlock}${bodyBlock}$response = ${cmd} -Uri "${url}" -Method ${method}${headerBlock ? ' -Headers $headers' : ''}${bodyBlock ? ' -Body $body' : ''}
${variantId === 'webrequest' ? '$response.Content' : '$response | ConvertTo-Json -Depth 5'}
`;
  }
};
