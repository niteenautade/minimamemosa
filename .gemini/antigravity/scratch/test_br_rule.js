const TurndownService = require('turndown');
const ts = new TurndownService({ headingStyle: 'atx' });
ts.addRule('br', {
  filter: 'br',
  replacement: function () {
    return '<br>';
  }
});
const html = "<p>Line 1<br><br><br>Line 2</p>";
console.log(ts.turndown(html));
