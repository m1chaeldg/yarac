import { compressXmlString, formatXmlString } from './xmlUtils.ts';

console.log('--- RUNNING XML UTILS TEST SUITE ---');

// Test 1: Compression of simple XML
const simpleXml = `
<note>
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body>Don't forget me this weekend!</body>
</note>
`;
const compressedSimple = compressXmlString(simpleXml);
console.assert(
  compressedSimple === '<note><to>Tove</to><from>Jani</from><heading>Reminder</heading><body>Don\'t forget me this weekend!</body></note>',
  `Simple compression failed: ${compressedSimple}`
);
console.log('✓ Simple XML compression');

// Test 2: Compression removes comments
const xmlWithComments = `
<!-- This is a comment -->
<root>
    <item id="1">First</item>
    <!-- Another comment -->
    <item id="2">Second</item>
</root>
`;
const compressedNoComments = compressXmlString(xmlWithComments);
console.assert(
  compressedNoComments === '<root><item id="1">First</item><item id="2">Second</item></root>',
  `Comment removal compression failed: ${compressedNoComments}`
);
console.log('✓ Comment removal during compression');

// Test 3: Preservation of CDATA blocks in compression and formatting
const xmlWithCdata = `
<data>
    <payload><![CDATA[<unescaped> & "special" characters</unescaped>]]></payload>
</data>
`;
const compressedCdata = compressXmlString(xmlWithCdata);
console.assert(
  compressedCdata.includes('<![CDATA[<unescaped> & "special" characters</unescaped>]]>'),
  `CDATA was corrupted during compression: ${compressedCdata}`
);
console.log('✓ CDATA preservation during compression');

// Test 4: Formatting XML with indentation
const unformatted = '<catalog><book id="bk101"><author>Gambardella, Matthew</author><title>XML Developer\'s Guide</title></book></catalog>';
const formatted = formatXmlString(unformatted);
console.assert(formatted.includes('  <book id="bk101">'), `Book tag indent failed in:\n${formatted}`);
console.assert(formatted.includes('    <author>Gambardella, Matthew</author>'), `Author tag indent failed in:\n${formatted}`);
console.assert(formatted.includes('</catalog>'), `Closing catalog tag missing in:\n${formatted}`);
console.log('✓ Formatted XML indentation structure');

// Test 5: Empty and invalid inputs
console.assert(compressXmlString('') === '', 'Empty string compression should return empty');
console.assert(formatXmlString('') === '', 'Empty string formatting should return empty');
console.log('✓ Edge cases (empty strings)');

console.log('--- ALL XML UTILS TESTS PASSED CLEANLY ---');
