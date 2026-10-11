import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const csharpLanguage: CodeLanguage = {
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
${body ? `request.AddStringBody(@"${body.replace(/"/g, '""')}", DataFormat.Json);\n` : ''}var response = await client.ExecuteAsync(request);
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
${body ? `request.Content = new StringContent(@"${body.replace(/"/g, '""')}", Encoding.UTF8, "${contentType}");\n` : ''}var response = await client.SendAsync(request);
var responseContent = await response.Content.ReadAsStringAsync();

Console.WriteLine($"Status: {(int)response.StatusCode} {response.ReasonPhrase}");
Console.WriteLine($"Body: {responseContent}");
`;
  }
};
