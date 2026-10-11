import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const javascriptLanguage: CodeLanguage = {
  id: 'javascript',
  name: 'JavaScript / Node',
  icon: '🟨',
  variants: [
    { id: 'fetch_browser', name: 'Browser Fetch' },
    { id: 'fetch_node', name: 'Node.js (Fetch API)' },
    { id: 'axios', name: 'Axios' }
  ],
  generate: (req, variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    if (variantId === 'axios') {
      return `import axios from 'axios';

const options = {
  method: '${method.toLowerCase()}',
  url: '${url}',
  headers: ${JSON.stringify(headers, null, 2)},
${body ? `  data: ${req.body.type === 'json' ? body : JSON.stringify(body)},\n` : ''}};

try {
  const { data, status } = await axios.request(options);
  console.log('Status:', status);
  console.log('Data:', data);
} catch (error) {
  console.error('Request failed:', error);
}
`;
    }

    // Default: Fetch API
    return `const response = await fetch('${url}', {
  method: '${method}',
  headers: ${JSON.stringify(headers, null, 2)},
${body ? `  body: ${req.body.type === 'json' ? `JSON.stringify(${body})` : JSON.stringify(body)},\n` : ''}});

const data = await response.text();
console.log('Status:', response.status);
console.log('Body:', data);
`;
  }
};
