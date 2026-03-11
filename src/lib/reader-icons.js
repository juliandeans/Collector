import {
  BookMarked,
  BookOpen,
  Briefcase,
  FileText,
  Files,
  FolderOpen,
  LibraryBig,
  Newspaper,
  NotebookText,
  Receipt,
  Scale,
  ScrollText,
} from "lucide-svelte";

export const readerIconOptions = [
  { id: "", label: "None", component: null },
  { id: "file-text", label: "File", component: FileText },
  { id: "book-open", label: "Book", component: BookOpen },
  { id: "book-marked", label: "Marked", component: BookMarked },
  { id: "newspaper", label: "Article", component: Newspaper },
  { id: "notebook-text", label: "Notebook", component: NotebookText },
  { id: "scroll-text", label: "Scroll", component: ScrollText },
  { id: "receipt", label: "Receipt", component: Receipt },
  { id: "scale", label: "Legal", component: Scale },
  { id: "briefcase", label: "Work", component: Briefcase },
  { id: "folder-open", label: "Folder", component: FolderOpen },
  { id: "files", label: "Stack", component: Files },
  { id: "library-big", label: "Library", component: LibraryBig },
];

export function getReaderIconComponent(iconId = "") {
  return readerIconOptions.find((option) => option.id === iconId)?.component ?? null;
}
