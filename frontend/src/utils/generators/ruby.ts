import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const rubyLanguage: CodeLanguage = {
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
${body ? `request.body = '${body.replace(/'/g, "\\'")}'\n` : ''}req_options = {
  use_ssl: uri.scheme == "https"
}

response = Net::HTTP.start(uri.hostname, uri.port, req_options) do |http|
  http.request(request)
end

puts "Status: #{response.code}"
puts response.body
`;
  }
};
