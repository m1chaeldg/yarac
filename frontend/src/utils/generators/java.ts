import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const javaLanguage: CodeLanguage = {
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
};
