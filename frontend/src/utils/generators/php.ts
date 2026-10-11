import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const phpLanguage: CodeLanguage = {
  id: 'php',
  name: 'PHP',
  icon: '🐘',
  variants: [
    { id: 'curl', name: 'PHP cURL' },
    { id: 'guzzle', name: 'Guzzle' }
  ],
  generate: (req, variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    if (variantId === 'guzzle') {
      return `<?php
require 'vendor/autoload.php';

$client = new \\GuzzleHttp\\Client();
$response = $client->request('${method}', '${url}', [
    'headers' => ${JSON.stringify(headers, null, 4)},
${body ? `    'body' => '${body.replace(/'/g, "\\'")}',\n` : ''}]);

echo "Status: " . $response->getStatusCode() . "\\n";
echo (string)$response->getBody();
`;
    }

    const headerArray = Object.entries(headers).map(([k, v]) => `    "${k}: ${v}"`).join(',\n');

    return `<?php
$curl = curl_init();

curl_setopt_array($curl, [
    CURLOPT_URL => "${url}",
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_CUSTOMREQUEST => "${method}",
${headerArray ? `    CURLOPT_HTTPHEADER => [\n${headerArray}\n    ],\n` : ''}${body ? `    CURLOPT_POSTFIELDS => '${body.replace(/'/g, "\\'")}',\n` : ''}]);

$response = curl_exec($curl);
$httpCode = curl_getinfo($curl, CURLINFO_HTTP_CODE);
curl_close($curl);

echo "Status: $httpCode\\n";
echo $response;
`;
  }
};
