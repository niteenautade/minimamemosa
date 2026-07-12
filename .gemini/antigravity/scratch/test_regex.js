const html = "<p>Line 1</p><p>&nbsp;</p><p>Line 2</p><p>&nbsp;</p><p>&nbsp;</p><p>&nbsp;</p><p>Line 3</p>";
const updated1 = html.replace(/<p([^>]*)>(\s*<br[^>]*\/?>)+\s*<\/p>/gi, '<p$1>MMEMPTY</p>');
const updated2 = updated1.replace(/<p([^>]*)>(?:\s|&nbsp;)*<\/p>/gi, '<p$1>MMEMPTY</p>');
console.log(updated2);
