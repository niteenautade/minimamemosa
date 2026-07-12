const html = "<p>Line 1<br><br><br>Line 2</p>";
const TurndownService = require('turndown');
const ts = new TurndownService({ headingStyle: 'atx' });
const md = ts.turndown(html);
console.log(JSON.stringify(md));
