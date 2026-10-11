import type { CodeLanguage } from './types';
import { curlLanguage } from './curl';
import { pythonLanguage } from './python';
import { javascriptLanguage } from './javascript';
import { csharpLanguage } from './csharp';
import { powershellLanguage } from './powershell';
import { goLanguage } from './go';
import { rustLanguage } from './rust';
import { shellLanguage } from './shell';
import { phpLanguage } from './php';
import { rubyLanguage } from './ruby';
import { javaLanguage } from './java';

export * from './types';
export * from './resolvers';
export {
  curlLanguage,
  pythonLanguage,
  javascriptLanguage,
  csharpLanguage,
  powershellLanguage,
  goLanguage,
  rustLanguage,
  shellLanguage,
  phpLanguage,
  rubyLanguage,
  javaLanguage
};

export const CODE_LANGUAGES: CodeLanguage[] = [
  curlLanguage,
  pythonLanguage,
  javascriptLanguage,
  csharpLanguage,
  powershellLanguage,
  goLanguage,
  rustLanguage,
  shellLanguage,
  phpLanguage,
  rubyLanguage,
  javaLanguage
];
