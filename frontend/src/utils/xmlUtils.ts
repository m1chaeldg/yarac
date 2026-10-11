/**
 * XML Utility functions for compression, minification, and pretty formatting.
 */

export function compressXmlString(xml: string): string {
  if (!xml) return '';
  // 1. Preserve CDATA blocks
  const cdataBlocks: string[] = [];
  let processed = xml.replace(/<!\[CDATA\[[\s\S]*?\]\]>/g, (match) => {
    cdataBlocks.push(match);
    return `___CDATA_${cdataBlocks.length - 1}___`;
  });

  // 2. Remove comments completely in compressed mode
  processed = processed.replace(/<!--[\s\S]*?-->/g, '');

  // 3. Remove leading/trailing newlines and spaces across whole document
  processed = processed.trim();

  // 4. Remove all whitespace between tags: >  \n  < -> ><
  processed = processed.replace(/>\s+</g, '><');

  // 5. Trim whitespace around text contents between tags: >  text  < -> >text<
  processed = processed.replace(/>\s*([^<]+?)\s*</g, (_, text) => {
    return `>${text.trim()}<`;
  });

  // 6. Normalize and collapse spaces/newlines inside opening tags & attributes
  processed = processed.replace(/<([^>!][^>]*?)>/g, (_, tagContent) => {
    return '<' + tagContent.replace(/\s+/g, ' ').trim() + '>';
  });

  // 7. Collapse any whitespace that was generated
  processed = processed.replace(/>\s+</g, '><');

  // 8. Restore CDATA blocks
  processed = processed.replace(/___CDATA_(\d+)___/g, (_, idx) => {
    return cdataBlocks[parseInt(idx, 10)] || '';
  });

  return processed.trim();
}

export function formatXmlString(xml: string): string {
  if (!xml) return '';
  const clean = compressXmlString(xml);
  let formatted = '';
  let pad = 0;
  const tokens = clean.match(/(<!\[CDATA\[[\s\S]*?\]\]>|<!--[\s\S]*?-->|<\?[^?]+\?>|<!DOCTYPE[^>]+>|<\/[^>]+>|<[^\/>]+(\/>|>)|[^<]+)/g) || [clean];

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i].trim();
    if (!token) continue;

    if (token.startsWith('</')) {
      pad = Math.max(0, pad - 1);
      formatted += '  '.repeat(pad) + token + '\n';
    } else if (token.startsWith('<') && !token.startsWith('<?') && !token.startsWith('<!') && !token.endsWith('/>') && !token.startsWith('<!--')) {
      const nextToken = tokens[i + 1]?.trim();
      const nextNextToken = tokens[i + 2]?.trim();
      const tagNameMatch = token.match(/<([^\s>]+)/);
      const tagName = tagNameMatch ? tagNameMatch[1] : '';

      // Inline short scalar text content
      if (nextToken && !nextToken.startsWith('<') && nextNextToken === `</${tagName}>`) {
        formatted += '  '.repeat(pad) + token + nextToken + nextNextToken + '\n';
        i += 2;
      } else {
        formatted += '  '.repeat(pad) + token + '\n';
        pad++;
      }
    } else {
      formatted += '  '.repeat(pad) + token + '\n';
    }
  }

  return formatted.trim();
}
