import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
import { common, createLowlight } from 'lowlight';
import Image from '@tiptap/extension-image';
import Link from '@tiptap/extension-link';

window.Tiptap = {
  Editor,
  StarterKit,
  Placeholder,
  Markdown,
  CodeBlockLowlight,
  lowlight: createLowlight(common),
  Image,
  Link
};
