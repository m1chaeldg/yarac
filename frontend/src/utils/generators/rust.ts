import type { CodeLanguage } from './types';
import { resolveUrl, resolveHeaders, resolveBody } from './resolvers';

export const rustLanguage: CodeLanguage = {
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
};
