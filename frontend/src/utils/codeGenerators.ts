import type { YaracRequest, KeyValuePair } from '../stores/workspace.svelte';

export interface CodeLanguage {
  id: string;
  name: string;
  icon: string;
  variants: Array<{ id: string; name: string }>;
  generate: (req: YaracRequest, variantId: string, envVars?: Record<string, string>) => string;
}

export function resolveUrl(req: YaracRequest, envVars?: Record<string, string>): string {
  let url = req.url || 'http://localhost:3000';
  if (envVars) {
    for (const [k, v] of Object.entries(envVars)) {
      url = url.split(`{{${k}}}`).join(v);
    }
  }

  // Active query parameters
  const activeParams = req.params.filter(p => p.enabled && !p.key.endsWith('~') && p.key.trim());
  if (activeParams.length > 0) {
    const separator = url.includes('?') ? '&' : '?';
    const qs = activeParams
      .map(p => {
        let val = p.value;
        if (envVars) {
          for (const [k, v] of Object.entries(envVars)) {
            val = val.split(`{{${k}}}`).join(v);
          }
        }
        return `${encodeURIComponent(p.key)}=${encodeURIComponent(val)}`;
      })
      .join('&');
    url = `${url}${separator}${qs}`;
  }

  return url;
}

export function resolveHeaders(req: YaracRequest, envVars?: Record<string, string>): Record<string, string> {
  const headers: Record<string, string> = {};

  for (const h of req.headers.filter(h => h.enabled && !h.key.endsWith('~') && h.key.trim())) {
    let val = h.value;
    if (envVars) {
      for (const [k, v] of Object.entries(envVars)) {
        val = val.split(`{{${k}}}`).join(v);
      }
    }
    headers[h.key] = val;
  }

  // Auth headers
  if (req.auth) {
    if (req.auth.type === 'bearer' && req.auth.bearerToken) {
      let token = req.auth.bearerToken;
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          token = token.split(`{{${k}}}`).join(v);
        }
      }
      headers['Authorization'] = `Bearer ${token}`;
    } else if (req.auth.type === 'basic' && (req.auth.basicUsername || req.auth.basicPassword)) {
      let u = req.auth.basicUsername || '';
      let p = req.auth.basicPassword || '';
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          u = u.split(`{{${k}}}`).join(v);
          p = p.split(`{{${k}}}`).join(v);
        }
      }
      const b64 = typeof btoa !== 'undefined' ? btoa(`${u}:${p}`) : Buffer.from(`${u}:${p}`).toString('base64');
      headers['Authorization'] = `Basic ${b64}`;
    } else if (req.auth.type === 'apikey' && req.auth.apiKeyLocation === 'header' && req.auth.apiKeyName) {
      let val = req.auth.apiKeyValue || '';
      if (envVars) {
        for (const [k, v] of Object.entries(envVars)) {
          val = val.split(`{{${k}}}`).join(v);
        }
      }
      headers[req.auth.apiKeyName] = val;
    }
  }

  // Auto content-type header for body if not present
  if (req.body) {
    if (req.body.type === 'json' && !headers['Content-Type']) {
      headers['Content-Type'] = 'application/json';
    } else if (req.body.type === 'form-urlencoded' && !headers['Content-Type']) {
      headers['Content-Type'] = 'application/x-www-form-urlencoded';
    }
  }

  return headers;
}

export function resolveBody(req: YaracRequest, envVars?: Record<string, string>): string {
  if (!req.body || req.body.type === 'none') return '';
  if (req.body.type === 'json' || req.body.type === 'raw') {
    let content = req.body.rawContent || '';
    if (envVars) {
      for (const [k, v] of Object.entries(envVars)) {
        content = content.split(`{{${k}}}`).join(v);
      }
    }
    return content;
  }
  if (req.body.type === 'form-urlencoded') {
    const active = req.body.formData.filter(f => f.enabled && !f.key.endsWith('~') && f.key.trim());
    return active
      .map(f => {
        let val = f.value;
        if (envVars) {
          for (const [k, v] of Object.entries(envVars)) {
            val = val.split(`{{${k}}}`).join(v);
          }
        }
        return `${encodeURIComponent(f.key)}=${encodeURIComponent(val)}`;
      })
      .join('&');
  }
  return '';
}

export const CODE_LANGUAGES: CodeLanguage[] = [
  {
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
  },
  {
    id: 'python',
    name: 'Python',
    icon: '🐍',
    variants: [
      { id: 'requests', name: 'Requests' },
      { id: 'httpx', name: 'HTTPX (Sync)' },
      { id: 'httpx_async', name: 'HTTPX (Async)' }
    ],
    generate: (req, variantId, envVars) => {
      const url = resolveUrl(req, envVars);
      const headers = resolveHeaders(req, envVars);
      const body = resolveBody(req, envVars);
      const method = req.method;

      const headerJson = JSON.stringify(headers, null, 4);

      if (variantId === 'httpx_async') {
        return `import asyncio
import httpx

async def main():
    url = "${url}"
    headers = ${headerJson}
${body ? `    payload = '''${body}'''\n` : ''}
    async with httpx.AsyncClient() as client:
        response = await client.request(
            "${method}",
            url,
            headers=headers,
${body ? `            content=payload,\n` : ''}            timeout=15.0
        )
        print("Status:", response.status_code)
        print("Response:", response.text)

if __name__ == "__main__":
    asyncio.run(main())
`;
      }

      if (variantId === 'httpx') {
        return `import httpx

url = "${url}"
headers = ${headerJson}
${body ? `payload = '''${body}'''\n` : ''}
with httpx.Client() as client:
    response = client.request(
        "${method}",
        url,
        headers=headers,
${body ? `        content=payload,\n` : ''}        timeout=15.0
    )
    print("Status:", response.status_code)
    print("Response:", response.text)
`;
      }

      // Default: requests
      return `import requests

url = "${url}"
headers = ${headerJson}
${body ? `payload = '''${body}'''\n` : ''}
response = requests.request(
    "${method}",
    url,
    headers=headers,
${body ? `    data=payload,\n` : ''}    timeout=15
)

print("Status:", response.status_code)
print("Response:", response.text)
`;
    }
  },
  {
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
  },
  {
    id: 'csharp',
    name: 'C#',
    icon: '🔷',
    variants: [
      { id: 'httpclient', name: 'HttpClient (.NET 6+)' },
      { id: 'restsharp', name: 'RestSharp' }
    ],
    generate: (req, variantId, envVars) => {
      const url = resolveUrl(req, envVars);
      const headers = resolveHeaders(req, envVars);
      const body = resolveBody(req, envVars);
      const method = req.method;

      if (variantId === 'restsharp') {
        const headerAdds = Object.entries(headers)
          .map(([k, v]) => `request.AddHeader("${k}", "${v}");`)
          .join('\n');

        return `using RestSharp;

var client = new RestClient("${url}");
var request = new RestRequest("", Method.${method === 'GET' ? 'Get' : method === 'POST' ? 'Post' : method === 'PUT' ? 'Put' : method === 'DELETE' ? 'Delete' : 'Patch'});
${headerAdds}
${body ? `request.AddStringBody(@"${body.replace(/"/g, '""')}", DataFormat.Json);\n` : ''}
var response = await client.ExecuteAsync(request);
Console.WriteLine($"Status: {response.StatusCode}");
Console.WriteLine($"Body: {response.Content}");
`;
      }

      // Default: HttpClient
      const headerAdds = Object.entries(headers)
        .filter(([k]) => k.toLowerCase() !== 'content-type')
        .map(([k, v]) => `request.Headers.Add("${k}", "${v}");`)
        .join('\n');

      const contentType = headers['Content-Type'] || 'application/json';

      return `using System.Net.Http;
using System.Text;

using var client = new HttpClient();
using var request = new HttpRequestMessage(HttpMethod.${method === 'GET' ? 'Get' : method === 'POST' ? 'Post' : method === 'PUT' ? 'Put' : method === 'DELETE' ? 'Delete' : 'Patch'}, "${url}");

${headerAdds}
${body ? `request.Content = new StringContent(@"${body.replace(/"/g, '""')}", Encoding.UTF8, "${contentType}");\n` : ''}
var response = await client.SendAsync(request);
var responseContent = await response.Content.ReadAsStringAsync();

Console.WriteLine($"Status: {(int)response.StatusCode} {response.ReasonPhrase}");
Console.WriteLine($"Body: {responseContent}");
`;
    }
  },
  {
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
  },
  {
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
  },
  {
    id: 'rust',
    name: 'Rust',
    icon: '🦀',
    variants: [
      { id: 'reqwest_async', name: 'reqwest (Async Tokio)' },
      { id: 'reqwest_blocking', name: 'reqwest (Blocking)' }
    ],
    generate: (req, variantId, envVars) => {
      const url = resolveUrl(req, envVars);
      const headers = resolveHeaders(req, envVars);
      const body = resolveBody(req, envVars);
      const method = req.method.toLowerCase();

      const headerChains = Object.entries(headers)
        .map(([k, v]) => `        .header("${k}", "${v}")`)
        .join('\n');

      if (variantId === 'reqwest_blocking') {
        return `// Add to Cargo.toml: reqwest = { version = "0.12", features = ["blocking"] }
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .${method}("${url}")
${headerChains}
${body ? `        .body(r#"${body}"#)\n` : ''}        .send()?;

    println!("Status: {}", res.status());
    println!("Body: {}", res.text()?);
    Ok(())
}
`;
      }

      return `// Add to Cargo.toml: reqwest = "0.12", tokio = { version = "1", features = ["full"] }
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .${method}("${url}")
${headerChains}
${body ? `        .body(r#"${body}"#)\n` : ''}        .send()
        .await?;

    println!("Status: {}", res.status());
    println!("Body: {}", res.text().await?);
    Ok(())
}
`;
    }
  },
  {
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
  },
  {
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
  },
  {
    id: 'ruby',
    name: 'Ruby',
    icon: '💎',
    variants: [
      { id: 'nethttp', name: 'Net::HTTP' }
    ],
    generate: (req, _variantId, envVars) => {
      const url = resolveUrl(req, envVars);
      const headers = resolveHeaders(req, envVars);
      const body = resolveBody(req, envVars);
      const method = req.method.charAt(0).toUpperCase() + req.method.slice(1).toLowerCase();

      const headerLines = Object.entries(headers)
        .map(([k, v]) => `request["${k}"] = "${v}"`)
        .join('\n');

      return `require 'net/http'
require 'uri'

uri = URI.parse("${url}")
request = Net::HTTP::${method}.new(uri)
${headerLines}
${body ? `request.body = '${body.replace(/'/g, "\\'")}'\n` : ''}
req_options = {
  use_ssl: uri.scheme == "https"
}

response = Net::HTTP.start(uri.hostname, uri.port, req_options) do |http|
  http.request(request)
end

puts "Status: #{response.code}"
puts response.body
`;
    }
  },
  {
    id: 'java',
    name: 'Java',
    icon: '☕',
    variants: [
      { id: 'httpclient', name: 'HttpClient (Java 11+)' }
    ],
    generate: (req, _variantId, envVars) => {
      const url = resolveUrl(req, envVars);
      const headers = resolveHeaders(req, envVars);
      const body = resolveBody(req, envVars);
      const method = req.method;

      const headerChains = Object.entries(headers)
        .map(([k, v]) => `                .header("${k}", "${v}")`)
        .join('\n');

      return `import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

public class App {
    public static void main(String[] args) throws Exception {
        HttpClient client = HttpClient.newHttpClient();

        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create("${url}"))
${headerChains}
${body ? `                .method("${method}", HttpRequest.BodyPublishers.ofString("${body.replace(/"/g, '\\"').replace(/\n/g, '\\n')}"))\n` : `                .method("${method}", HttpRequest.BodyPublishers.noBody())\n`}                .build();

        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());
        System.out.println("Status: " + response.statusCode());
        System.out.println(response.body());
    }
}
`;
    }
  }
];

export function getRelativeRequestPath(collectionName: string, requestName: string): string {
  const sanitize = (s: string) => s.toLowerCase().replace(/[^a-z0-9_-]+/g, '_').replace(/^_+|_+$/g, '');
  const c = sanitize(collectionName) || 'general';
  const r = sanitize(requestName) || 'request';
  return `collections/${c}/${r}.yarac`;
}
