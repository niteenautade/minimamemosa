const { JSDOM } = require("jsdom");
const dom = new JSDOM(`
<!DOCTYPE html>
<body>
  <div id="memo-editor"></div>
  <script type="module">
    import { Editor } from 'https://cdn.jsdelivr.net/npm/@tiptap/core@2.6.6/+esm';
    import { StarterKit } from 'https://cdn.jsdelivr.net/npm/@tiptap/starter-kit@2.6.6/+esm';
    import { Markdown } from 'https://cdn.jsdelivr.net/npm/tiptap-markdown@0.8.10/+esm';
    
    console.log("Imports succeeded", !!Editor, !!StarterKit, !!Markdown);
    const editor = new Editor({
      element: document.getElementById('memo-editor'),
      extensions: [ StarterKit, Markdown ]
    });
    console.log("Editor init succeeded");
  </script>
</body>
`);
// wait for it
