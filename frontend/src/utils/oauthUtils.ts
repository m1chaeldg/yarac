/**
 * OAuth 2.0 & PKCE (Proof Key for Code Exchange) RFC 7636 utilities.
 * 100% offline-native, using standard Web Cryptography APIs.
 */

export interface OAuth2TokenResponse {
  access_token: string;
  token_type?: string;
  expires_in?: number;
  refresh_token?: string;
  scope?: string;
  error?: string;
  error_description?: string;
}

/**
 * Generates a high-entropy cryptographic random code_verifier for PKCE.
 * Allowed characters: [A-Z], [a-z], [0-9], "-", ".", "_", "~".
 */
export function generateCodeVerifier(length = 64): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomBytes = new Uint8Array(length);
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
    crypto.getRandomValues(randomBytes);
  } else {
    for (let i = 0; i < length; i++) {
      randomBytes[i] = Math.floor(Math.random() * 256);
    }
  }
  let result = '';
  for (let i = 0; i < length; i++) {
    result += charset[randomBytes[i] % charset.length];
  }
  return result;
}

/**
 * Calculates the SHA-256 base64url-encoded code_challenge from a code_verifier.
 */
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const base64 = btoa(String.fromCharCode.apply(null, hashArray));
  // Convert base64 to base64url format
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * Builds the authorization URL for the Authorization Code flow (with optional PKCE).
 */
export function buildAuthorizationUrl(params: {
  authUrl: string;
  clientId: string;
  redirectUri: string;
  scope?: string;
  state?: string;
  codeChallenge?: string;
  codeChallengeMethod?: 'S256' | 'plain';
}): string {
  const url = new URL(params.authUrl);
  url.searchParams.set('response_type', 'code');
  url.searchParams.set('client_id', params.clientId);
  url.searchParams.set('redirect_uri', params.redirectUri);
  if (params.scope) {
    url.searchParams.set('scope', params.scope);
  }
  if (params.state) {
    url.searchParams.set('state', params.state);
  }
  if (params.codeChallenge) {
    url.searchParams.set('code_challenge', params.codeChallenge);
    url.searchParams.set('code_challenge_method', params.codeChallengeMethod || 'S256');
  }
  return url.toString();
}

/**
 * Exchanges client credentials or an authorization code for an OAuth 2.0 access token.
 */
export async function exchangeOAuthToken(params: {
  tokenUrl: string;
  grantType: 'authorization_code' | 'client_credentials';
  clientId: string;
  clientSecret?: string;
  code?: string;
  redirectUri?: string;
  codeVerifier?: string;
  scope?: string;
}): Promise<OAuth2TokenResponse> {
  const bodyParams = new URLSearchParams();
  bodyParams.set('grant_type', params.grantType);
  bodyParams.set('client_id', params.clientId);

  if (params.clientSecret) {
    bodyParams.set('client_secret', params.clientSecret);
  }
  if (params.scope) {
    bodyParams.set('scope', params.scope);
  }

  if (params.grantType === 'authorization_code') {
    if (params.code) bodyParams.set('code', params.code);
    if (params.redirectUri) bodyParams.set('redirect_uri', params.redirectUri);
    if (params.codeVerifier) bodyParams.set('code_verifier', params.codeVerifier);
  }

  const headers: Record<string, string> = {
    'Content-Type': 'application/x-www-form-urlencoded',
    'Accept': 'application/json'
  };

  const res = await fetch(params.tokenUrl, {
    method: 'POST',
    headers,
    body: bodyParams.toString()
  });

  if (!res.ok) {
    const errorText = await res.text();
    try {
      const errJson = JSON.parse(errorText);
      throw new Error(errJson.error_description || errJson.error || `HTTP ${res.status}`);
    } catch {
      throw new Error(`Token request failed with status ${res.status}: ${errorText}`);
    }
  }

  return await res.json();
}
