import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { marked } from "marked";
import { listen } from "@tauri-apps/api/event";

const openVaultBtn = document.getElementById(
  "open-vault-btn",
) as HTMLButtonElement;
const openFileBtn = document.getElementById(
  "open-file-btn",
) as HTMLButtonElement;
const sortToggleBtn = document.getElementById(
  "sort-toggle-btn",
) as HTMLButtonElement;
const fileList = document.getElementById("file-list") as HTMLElement;
const preview = document.getElementById("preview") as HTMLDivElement;

let isDescending = false;
let currentDir = "";

openVaultBtn.addEventListener("click", async () => {
  const dir = await open({ directory: true });
  if (typeof dir === "string") {
    currentDir = dir;
    await loadFiles();
  }
});

async function loadFiles() {
  if (!currentDir) return;
  
  const files = await invoke<string[]>("list_markdown_files", { 
    dir: currentDir, 
    descending: isDescending 
  });
  fileList.innerHTML = "";
  for (const file of files) {
    const li = document.createElement("li");
    const name = file.split("\\").pop() ?? file;
    li.textContent = name;
    li.title = file;
    li.style.cursor = "pointer";
    li.addEventListener("click", async () => {
      const content = await invoke<string>("read_file", { path: file });
      preview.innerHTML = await marked(content);
    });
    fileList.appendChild(li);
  }
}

sortToggleBtn.addEventListener("click", async () => {
  isDescending = !isDescending;
  sortToggleBtn.textContent = isDescending ? "ソート順: ▲" : "ソート順: ▼";
  await loadFiles();
});

//個別ファイルを開く
openFileBtn.addEventListener("click", async () => {
  const path = await open({
    filters: [{ name: "Markdown", extensions: ["md"] }],
  });
  if (typeof path === "string") {
    const content = await invoke<string>("read_file", { path });
    preview.innerHTML = await marked(content);
  }
});

//Drag and Drop
(async () => {
  await listen<string>("file-dropped", async (event) => {
    const path = event.payload;
    const content = await invoke<string>("read_file", { path });
    preview.innerHTML = await marked(content);
  });
})();
