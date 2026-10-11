import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const pythonLanguage: CodeLanguage = {
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
${body ? `    payload = '''${body}'''\n` : ''}    async with httpx.AsyncClient() as client:
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
${body ? `payload = '''${body}'''\n` : ''}with httpx.Client() as client:
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
${body ? `payload = '''${body}'''\n` : ''}response = requests.request(
    "${method}",
    url,
    headers=headers,
${body ? `    data=payload,\n` : ''}    timeout=15
)

print("Status:", response.status_code)
print("Response:", response.text)
`;
  }
};
