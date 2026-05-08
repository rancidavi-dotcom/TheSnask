;(function () {
  "use strict";

  // ─── Dark Mode ───
  function applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("snask-theme", theme);
  }

  function initTheme() {
    var stored = localStorage.getItem("snask-theme");
    if (stored === "dark" || stored === "light") {
      applyTheme(stored);
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      applyTheme("dark");
    } else {
      applyTheme("light");
    }
  }

  function injectThemeToggle() {
    var topbar = document.querySelector(".topbar");
    if (!topbar) return;

    var btn = document.createElement("button");
    btn.className = "theme-toggle";
    btn.setAttribute("aria-label", "Alternar tema");
    btn.innerHTML = document.documentElement.getAttribute("data-theme") === "dark" ? "☀️" : "🌙";

    // Insert before nav or at end
    var nav = topbar.querySelector(".nav");
    if (nav) {
      nav.parentNode.insertBefore(btn, nav.nextSibling);
    } else {
      topbar.appendChild(btn);
    }

    btn.addEventListener("click", function () {
      var current = document.documentElement.getAttribute("data-theme");
      var next = current === "dark" ? "light" : "dark";
      applyTheme(next);
      btn.innerHTML = next === "dark" ? "☀️" : "🌙";
    });
  }

  // ─── Hamburger ───
  function injectHamburger() {
    var topbar = document.querySelector(".topbar");
    var nav = topbar && topbar.querySelector(".nav");
    if (!topbar || !nav) return;

    var btn = document.createElement("button");
    btn.className = "hamburger";
    btn.setAttribute("aria-label", "Abrir menu");
    for (var i = 0; i < 3; i++) {
      btn.appendChild(document.createElement("span"));
    }

    // Insert at start of topbar (before brand)
    topbar.insertBefore(btn, topbar.firstChild);

    btn.addEventListener("click", function () {
      nav.classList.toggle("open");
    });
  }

  // ─── Sidebar Active Link ───
  function highlightSidebar() {
    var links = document.querySelectorAll(".sidebar a");
    var current = location.pathname.split("/").pop() || "index.html";

    for (var i = 0; i < links.length; i++) {
      var href = links[i].getAttribute("href") || "";
      if (href.endsWith(current)) {
        links[i].classList.add("active");
      }
    }
  }

  // ─── Copy Buttons ───
  function addCopyButtons() {
    var blocks = document.querySelectorAll("pre");
    for (var i = 0; i < blocks.length; i++) {
      var block = blocks[i];
      if (block.querySelector(".copy-btn")) continue;

      var codeEl = block.querySelector("code");
      var getText = function () {
        return codeEl ? codeEl.innerText : block.innerText;
      };

      var btn = document.createElement("button");
      btn.className = "copy-btn";
      btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> Copiar';

      var fallbackCopy = function (text) {
        try {
          var ta = document.createElement("textarea");
          ta.value = text;
          ta.setAttribute("readonly", "");
          ta.style.position = "fixed";
          ta.style.left = "-9999px";
          document.body.appendChild(ta);
          ta.select();
          var r = document.execCommand("copy");
          document.body.removeChild(ta);
          return r;
        } catch (e) { return false; }
      };

      btn.addEventListener("click", function () {
        var text = getText();
        var copied = false;
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(function () {
            copied = true;
          }).catch(function () {
            copied = fallbackCopy(text);
          }).finally(function () {
            btn.innerHTML = copied
              ? '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg> Copiado'
              : "Falha";
            setTimeout(function () { btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> Copiar'; }, 1500);
          });
        } else {
          copied = fallbackCopy(text);
          btn.textContent = copied ? "Copiado" : "Falha";
          setTimeout(function () { btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> Copiar'; }, 1500);
        }
      });

      block.appendChild(btn);
    }
  }

  // ─── Functions Search ───
  function initFunctionSearch() {
    var input = document.getElementById("functionSearch");
    if (!input) return;

    var cards = document.querySelectorAll(".function-card");
    input.addEventListener("input", function () {
      var q = input.value.toLowerCase();
      for (var i = 0; i < cards.length; i++) {
        var card = cards[i];
        var text = card.textContent.toLowerCase();
        card.style.display = text.indexOf(q) > -1 ? "" : "none";
      }
    });
  }

  // ─── Init ───
  document.addEventListener("DOMContentLoaded", function () {
    initTheme();
    injectThemeToggle();
    injectHamburger();
    highlightSidebar();
    addCopyButtons();
    initFunctionSearch();
  });
})();
