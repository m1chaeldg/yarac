import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const goLanguage: CodeLanguage = {
  id: 'go',
  name: 'Go',
  icon: '🐹',
  variants: [
    { id: 'nethttp', name: 'net/http' }
  ],
  generate: (req, _variantId, envVars) => {
    const url = resolveUrl(req, envVars);
    const headers = resolveHeaders(req, envVars);
    const body = resolveBody(req, envVars);
    const method = req.method;

    const headerLines = Object.entries(headers)
      .map(([k, v]) => `\treq.Header.Set("${k}", "${v}")`)
      .join('\n');

    return `package main

import (
\t"bytes"
\t"fmt"
\t"io"
\t"net/http"
)

func main() {
\turl := "${url}"
${body ? `\tbody := []byte(\`${body}\`)\n\treq, err := http.NewRequest("${method}", url, bytes.NewBuffer(body))\n` : `\treq, err := http.NewRequest("${method}", url, nil)\n`}\tif err != nil {
\t\tpanic(err)
\t}

${headerLines}

\tresp, err := http.DefaultClient.Do(req)
\tif err != nil {
\t\tpanic(err)
\t}
\tdefer resp.Body.Close()

\trespBody, err := io.ReadAll(resp.Body)
\tif err != nil {
\t\tpanic(err)
\t}

\tfmt.Printf("Status: %d\\n", resp.StatusCode)
\tfmt.Println(string(respBody))
}
`;
  }
};
