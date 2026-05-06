const links = document.querySelectorAll(".sidebar a");
const current = location.pathname.split("/").pop() || "index.html";

for (const link of links) {
  const href = link.getAttribute("href") || "";
  if (href.endsWith(current)) {
    link.classList.add("active");
  }
}

for (const block of document.querySelectorAll("pre")) {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = "Copiar";
  button.style.float = "right";
  button.style.margin = "0 0 8px 12px";
  button.style.border = "1px solid rgba(255,255,255,.25)";
  button.style.borderRadius = "6px";
  button.style.background = "rgba(255,255,255,.08)";
  button.style.color = "white";
  button.style.padding = "4px 8px";
  button.style.cursor = "pointer";
  button.addEventListener("click", async () => {
    await navigator.clipboard.writeText(block.innerText.replace(/^Copiar\s*/, ""));
    button.textContent = "Copiado";
    setTimeout(() => (button.textContent = "Copiar"), 900);
  });
  block.prepend(button);
}
