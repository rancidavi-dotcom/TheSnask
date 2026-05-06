document.addEventListener("DOMContentLoaded", () => {
  const links = document.querySelectorAll(".sidebar a");
  const current = location.pathname.split("/").pop() || "index.html";

  for (const link of links) {
    const href = link.getAttribute("href") || "";
    if (href.endsWith(current)) {
      link.classList.add("active");
    }
  }

  for (const block of document.querySelectorAll("pre")) {
    const codeEl = block.querySelector("code");
    const getText = () => (codeEl ? codeEl.innerText : block.innerText);

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

    const fallbackCopy = (text) => {
      try {
        const ta = document.createElement("textarea");
        ta.value = text;
        ta.setAttribute("readonly", "");
        ta.style.position = "fixed";
        ta.style.left = "-9999px";
        document.body.appendChild(ta);
        ta.select();
        const successful = document.execCommand("copy");
        document.body.removeChild(ta);
        return successful;
      } catch (err) {
        console.warn("Fallback copy failed", err);
        return false;
      }
    };

    button.addEventListener("click", async () => {
      const text = getText();
      let copied = false;
      if (navigator.clipboard && navigator.clipboard.writeText) {
        try {
          await navigator.clipboard.writeText(text);
          copied = true;
        } catch (e) {
          copied = fallbackCopy(text);
        }
      } else {
        copied = fallbackCopy(text);
      }

      button.textContent = copied ? "Copiado" : "Falha";
      setTimeout(() => (button.textContent = "Copiar"), 900);
    });

    block.prepend(button);
  }
});
