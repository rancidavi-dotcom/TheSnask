#!/usr/bin/env python3
# Snask Store — GUI para instalar/remover/atualizar pacotes do registry Snask.

import json
import os
import sys
import threading
import subprocess
import urllib.request
import urllib.error
from dataclasses import dataclass
from typing import Dict, Optional, Tuple, List


REGISTRY_GIT_URL = "https://github.com/rancidavi-dotcom/SnaskPackages"
REGISTRY_HTTP_FALLBACK_URL = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/registry.json"
BASE_PKG_URL = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/packages/"
# PyInstaller: assets ficam dentro de sys._MEIPASS
_BASE_DIR = getattr(sys, "_MEIPASS", os.path.dirname(__file__))
ASSETS_DIR = os.path.join(_BASE_DIR, "assets")
ICONS_DIR = os.path.join(ASSETS_DIR, "icons")
APP_ICON = os.path.join(ASSETS_DIR, "snask.svg")
REGISTRY_LOCAL_DIR = os.path.join(os.path.expanduser("~"), ".snask", "registry")


def home_dir() -> str:
    return os.path.expanduser("~")


def packages_dir() -> str:
    return os.path.join(home_dir(), ".snask", "packages")


def ensure_packages_dir() -> None:
    os.makedirs(packages_dir(), exist_ok=True)


def http_get_text(url: str, timeout_s: int = 20) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "SnaskStore/0.1"})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        data = resp.read()
    return data.decode("utf-8", errors="replace")

def run_git(args: List[str], cwd: Optional[str] = None) -> Tuple[bool, str]:
    try:
        p = subprocess.run(["git"] + args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        if p.returncode != 0:
            return False, (p.stderr or p.stdout or "").strip()
        return True, ""
    except Exception as e:
        return False, str(e)


def ensure_registry_repo() -> Tuple[bool, str]:
    os.makedirs(os.path.dirname(REGISTRY_LOCAL_DIR), exist_ok=True)
    if os.path.isdir(os.path.join(REGISTRY_LOCAL_DIR, ".git")):
        ok, err = run_git(["fetch", "--all", "--prune"], cwd=REGISTRY_LOCAL_DIR)
        if not ok:
            return False, err
        ok, err = run_git(["pull", "--ff-only"], cwd=REGISTRY_LOCAL_DIR)
        if not ok:
            return False, err
        return True, ""

    ok, err = run_git(["clone", "--depth", "1", REGISTRY_GIT_URL, REGISTRY_LOCAL_DIR])
    if not ok:
        return False, err
    return True, ""


def load_registry_json_from_git() -> Tuple[Optional[dict], Optional[str]]:
    ok, err = ensure_registry_repo()
    if not ok:
        return None, err

    index_dir = os.path.join(REGISTRY_LOCAL_DIR, "index")
    if os.path.isdir(index_dir):
        pkgs = {}
        for root, _dirs, files in os.walk(index_dir):
            for fn in files:
                if not fn.endswith(".json"):
                    continue
                name = os.path.splitext(fn)[0]
                path = os.path.join(root, fn)
                try:
                    with open(path, "r", encoding="utf-8") as f:
                        meta = json.load(f)
                    if isinstance(meta, dict):
                        pkgs[name] = meta
                except Exception:
                    continue
        return {"packages": pkgs}, None

    legacy = os.path.join(REGISTRY_LOCAL_DIR, "registry.json")
    if os.path.isfile(legacy):
        try:
            with open(legacy, "r", encoding="utf-8") as f:
                return json.load(f), None
        except Exception as e:
            return None, str(e)

    return None, "registry inválido (sem index/ e sem registry.json)"


def load_registry_json_from_http() -> Tuple[Optional[dict], Optional[str]]:
    try:
        raw = http_get_text(REGISTRY_HTTP_FALLBACK_URL)
        return json.loads(raw), None
    except Exception as e:
        return None, str(e)


@dataclass(frozen=True)
class Package:
    name: str
    version: str
    url: str
    description: str

    def file_name(self) -> str:
        u = (self.url or "").strip()
        if not u:
            return f"{self.name}.snask"
        if u.endswith(".snask"):
            return os.path.basename(u)
        return f"{self.name}.snask"

    def download_url(self) -> str:
        u = (self.url or "").strip()
        if not u:
            return f"{BASE_PKG_URL}{self.name}.snask"
        if u.startswith("http://") or u.startswith("https://"):
            return u
        return f"{BASE_PKG_URL}{u}"

    def installed_path(self) -> str:
        return os.path.join(packages_dir(), self.file_name())

    def is_installed(self) -> bool:
        return os.path.exists(self.installed_path())


def load_registry() -> Dict[str, Package]:
    print("[store] baixando registry (git)…")
    obj, err = load_registry_json_from_git()
    if obj is None:
        print(f"[store] git falhou: {err} (fallback HTTP)")
        obj, err2 = load_registry_json_from_http()
        if obj is None:
            raise RuntimeError(f"registry inválido: {err2}")

    pkgs = obj.get("packages", {})
    out: Dict[str, Package] = {}
    for name, meta in pkgs.items():
        out[name] = Package(
            name=name,
            version=str(meta.get("version", "unknown")),
            url=str(meta.get("url", "")),
            description=str(meta.get("description", "")),
        )
    return out


def install_package(pkg: Package) -> None:
    ensure_packages_dir()
    body = http_get_text(pkg.download_url())
    with open(pkg.installed_path(), "w", encoding="utf-8") as f:
        f.write(body)


def uninstall_package(pkg: Package) -> None:
    p = pkg.installed_path()
    if os.path.exists(p):
        os.remove(p)


def update_package(pkg: Package) -> None:
    install_package(pkg)


def fmt_pkg_details(pkg: Package) -> str:
    installed = "sim" if pkg.is_installed() else "não"
    return (
        f"Pacote: {pkg.name}\n"
        f"Versão: {pkg.version}\n"
        f"Instalado: {installed}\n"
        f"Arquivo: {pkg.installed_path()}\n"
        f"URL: {pkg.download_url()}\n\n"
        f"{pkg.description}"
    )


def pkg_icon_path(pkg_name: str) -> Optional[str]:
    p = os.path.join(ICONS_DIR, f"{pkg_name}.svg")
    if os.path.exists(p):
        return p
    return None


def main() -> int:
    try:
        import gi  # type: ignore
        gi.require_version("Gtk", "3.0")
        from gi.repository import Gtk, GLib, Gdk  # type: ignore
    except Exception as e:
        print("Snask Store: dependência ausente para GUI.")
        print("Instale:", "sudo apt install -y python3-gi gir1.2-gtk-3.0")
        print("Erro:", e)
        return 1

    CSS = b"""
    .snask-title { font-weight: 700; font-size: 16px; }
    .snask-subtle { opacity: 0.75; }
    .snask-badge {
        padding: 2px 8px;
        border-radius: 999px;
        border: 1px solid alpha(@theme_fg_color, 0.18);
    }
    .snask-badge.installed {
        background: alpha(@theme_selected_bg_color, 0.18);
        border-color: alpha(@theme_selected_bg_color, 0.35);
    }
    .snask-hero { font-weight: 700; font-size: 18px; }
    .snask-card {
        border-radius: 12px;
        border: 1px solid alpha(@theme_fg_color, 0.10);
        background: alpha(@theme_base_color, 1.0);
    }
    """

    def apply_css() -> None:
        try:
            provider = Gtk.CssProvider()
            provider.load_from_data(CSS)
            screen = Gdk.Screen.get_default()
            if screen is not None:
                Gtk.StyleContext.add_provider_for_screen(screen, provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
        except Exception:
            pass

    class PkgCard(Gtk.FlowBoxChild):
        def __init__(self, pkg: Package) -> None:
            super().__init__()
            self.pkg = pkg

            frame = Gtk.Frame()
            frame.set_shadow_type(Gtk.ShadowType.IN)
            frame.get_style_context().add_class("snask-card")

            outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
            outer.set_border_width(12)
            frame.add(outer)

            top = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
            outer.pack_start(top, False, False, 0)

            icon_p = pkg_icon_path(pkg.name)
            if icon_p:
                icon = Gtk.Image.new_from_file(icon_p)
            else:
                icon = Gtk.Image.new_from_icon_name("package-x-generic", Gtk.IconSize.DIALOG)
            top.pack_start(icon, False, False, 0)

            title_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            top.pack_start(title_box, True, True, 0)

            title = Gtk.Label(label=pkg.name)
            title.set_xalign(0.0)
            title.get_style_context().add_class("snask-title")
            title_box.pack_start(title, False, False, 0)

            ver = Gtk.Label(label=f"v{pkg.version}")
            ver.set_xalign(0.0)
            ver.get_style_context().add_class("snask-subtle")
            title_box.pack_start(ver, False, False, 0)

            badge = Gtk.Label(label=("Instalado" if pkg.is_installed() else ""))
            badge.set_xalign(1.0)
            badge.get_style_context().add_class("snask-badge")
            if pkg.is_installed():
                badge.get_style_context().add_class("installed")
            top.pack_start(badge, False, False, 0)

            desc = Gtk.Label(label=(pkg.description or ""))
            desc.set_xalign(0.0)
            desc.set_line_wrap(True)
            desc.set_max_width_chars(44)
            desc.get_style_context().add_class("snask-subtle")
            outer.pack_start(desc, True, True, 0)
            self.set_tooltip_text(pkg.description or "")

            actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            outer.pack_start(actions, False, False, 0)

            self.btn_install = Gtk.Button(label="Instalar")
            self.btn_remove = Gtk.Button(label="Remover")
            self.btn_update = Gtk.Button(label="Atualizar")
            actions.pack_start(self.btn_install, True, True, 0)
            actions.pack_start(self.btn_remove, True, True, 0)
            actions.pack_start(self.btn_update, True, True, 0)

            self.badge = badge

            self.add(frame)
            self.refresh_buttons()

        def refresh_buttons(self) -> None:
            installed = self.pkg.is_installed()
            self.btn_install.set_sensitive(not installed)
            self.btn_remove.set_sensitive(installed)
            self.btn_update.set_sensitive(True)
            if installed:
                self.badge.set_text("Instalado")
                self.badge.get_style_context().add_class("installed")
            else:
                self.badge.set_text("")
                self.badge.get_style_context().remove_class("installed")

        def open_description_dialog(self, parent) -> None:
            d = Gtk.MessageDialog(
                transient_for=parent,
                flags=0,
                message_type=Gtk.MessageType.INFO,
                buttons=Gtk.ButtonsType.CLOSE,
                text=self.pkg.name,
            )
            d.format_secondary_text(self.pkg.description or "(sem descrição)")
            d.run()
            d.destroy()

    class App(Gtk.Window):
        def __init__(self) -> None:
            super().__init__(title="Snask Store")
            self.set_default_size(980, 600)
            self.connect("destroy", Gtk.main_quit)
            if os.path.exists(APP_ICON):
                try:
                    self.set_icon_from_file(APP_ICON)
                except Exception:
                    pass

            self.registry: Dict[str, Package] = {}
            self._packages_mtime: float = 0.0

            root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
            self.add(root)

            # HeaderBar (Discover-like)
            hb = Gtk.HeaderBar()
            hb.set_show_close_button(True)
            hb.props.title = "Snask Store"
            hb.props.subtitle = "Pacotes para Snask"
            self.set_titlebar(hb)

            self.refresh_btn = Gtk.Button.new_from_icon_name("view-refresh-symbolic", Gtk.IconSize.BUTTON)
            self.refresh_btn.set_tooltip_text("Atualizar registry")
            self.refresh_btn.connect("clicked", self.on_refresh)
            hb.pack_start(self.refresh_btn)

            self.spinner = Gtk.Spinner()
            hb.pack_start(self.spinner)

            self.search = Gtk.SearchEntry()
            self.search.set_placeholder_text("Buscar pacotes (ex: json, os, gui)")
            self.search.connect("search-changed", self.on_search_changed)
            hb.pack_end(self.search)

            # Sub-header status
            status_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            status_bar.set_border_width(10)
            root.pack_start(status_bar, False, False, 0)

            self.status_lbl = Gtk.Label(label="Carregando registry…")
            self.status_lbl.set_xalign(0.0)
            self.status_lbl.get_style_context().add_class("snask-subtle")
            status_bar.pack_start(self.status_lbl, True, True, 0)

            # Main: scroll feed (Play Store-like)
            sc = Gtk.ScrolledWindow()
            sc.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)
            root.pack_start(sc, True, True, 0)

            self.flow = Gtk.FlowBox()
            self.flow.set_selection_mode(Gtk.SelectionMode.NONE)
            self.flow.set_max_children_per_line(3)
            self.flow.set_row_spacing(12)
            self.flow.set_column_spacing(12)
            self.flow.set_homogeneous(False)
            self.flow.connect("child-activated", self.on_card_activated)

            grid_wrap = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
            grid_wrap.set_border_width(12)
            sc.add(grid_wrap)

            hero = Gtk.Label(label="Descubra pacotes para Snask")
            hero.set_xalign(0.0)
            hero.get_style_context().add_class("snask-hero")
            grid_wrap.pack_start(hero, False, False, 0)

            sub = Gtk.Label(label=f"Instala em {packages_dir()}")
            sub.set_xalign(0.0)
            sub.get_style_context().add_class("snask-subtle")
            grid_wrap.pack_start(sub, False, False, 0)

            grid_wrap.pack_start(self.flow, True, True, 0)

            self.cards: List[PkgCard] = []

            self.load_registry_async()
            # Mantém a UI sempre consistente: se algo mexer em ~/.snask/packages (CLI ou GUI),
            # atualiza os badges e botões automaticamente.
            GLib.timeout_add_seconds(2, self.poll_installed_state)
            # Atualiza registry periodicamente (mantém catálogo em dia).
            GLib.timeout_add_seconds(120, self.poll_registry_refresh)

        def on_search_changed(self, _w) -> None:
            self.apply_filter()

        def on_refresh(self, _w) -> None:
            self.load_registry_async()

        def load_registry_async(self) -> None:
            self.status_lbl.set_text("Baixando registry.json…")
            self.refresh_btn.set_sensitive(False)
            self.spinner.start()

            def work() -> Tuple[Optional[Dict[str, Package]], Optional[str]]:
                try:
                    reg = load_registry()
                    return reg, None
                except urllib.error.URLError as e:
                    return None, f"Falha de rede: {e}"
                except Exception as e:
                    return None, f"Erro: {e}"

            def done(result: Tuple[Optional[Dict[str, Package]], Optional[str]]) -> None:
                reg, err = result
                self.refresh_btn.set_sensitive(True)
                self.spinner.stop()
                if err:
                    self.status_lbl.set_text(err)
                    for child in list(self.flow.get_children()):
                        self.flow.remove(child)
                    self.cards = []
                    return
                assert reg is not None
                self.registry = reg
                for child in list(self.flow.get_children()):
                    self.flow.remove(child)
                self.cards = []
                for name in sorted(self.registry.keys()):
                    c = PkgCard(self.registry[name])
                    c.btn_install.connect("clicked", self.on_install_card, c)
                    c.btn_remove.connect("clicked", self.on_remove_card, c)
                    c.btn_update.connect("clicked", self.on_update_card, c)
                    self.flow.add(c)
                    self.cards.append(c)
                self.flow.show_all()
                self.apply_filter()
                self.status_lbl.set_text(f"OK ({len(self.registry)} pacotes) • instala em {packages_dir()}")

            self.run_bg(work, done)

        def on_card_activated(self, _flow, child) -> None:
            if isinstance(child, PkgCard):
                child.open_description_dialog(self)

        def apply_filter(self) -> None:
            q = (self.search.get_text() or "").strip().lower()
            shown = 0
            for row in self.cards:
                visible = True
                if q:
                    visible = q in row.pkg.name.lower() or q in (row.pkg.description or "").lower()
                row.set_visible(visible)
                if visible:
                    shown += 1
            if self.registry:
                self.status_lbl.set_text(f"Mostrando {shown}/{len(self.registry)} • {packages_dir()}")

        def on_install_card(self, _w, card: PkgCard) -> None:
            pkg = card.pkg
            self.status_lbl.set_text(f"Instalando {pkg.name}…")
            self.spinner.start()

            def work():
                install_package(pkg)
                return None, None

            def done(_):
                self.spinner.stop()
                self.status_lbl.set_text(f"Instalado: {pkg.name}")
                self.refresh_rows()
                self.load_registry_async()

            self.run_bg(work, done)

        def on_remove_card(self, _w, card: PkgCard) -> None:
            pkg = card.pkg
            self.status_lbl.set_text(f"Removendo {pkg.name}…")
            self.spinner.start()

            def work():
                uninstall_package(pkg)
                return None, None

            def done(_):
                self.spinner.stop()
                self.status_lbl.set_text(f"Removido: {pkg.name}")
                self.refresh_rows()
                self.load_registry_async()

            self.run_bg(work, done)

        def on_update_card(self, _w, card: PkgCard) -> None:
            pkg = card.pkg
            self.status_lbl.set_text(f"Atualizando {pkg.name}…")
            self.spinner.start()

            def work():
                update_package(pkg)
                return None, None

            def done(_):
                self.spinner.stop()
                self.status_lbl.set_text(f"Atualizado: {pkg.name}")
                self.refresh_rows()
                self.load_registry_async()

            self.run_bg(work, done)

        def refresh_rows(self) -> None:
            for row in self.cards:
                row.refresh_buttons()
            self.apply_filter()

        def poll_installed_state(self) -> bool:
            # Atualiza sempre: é barato e garante consistência mesmo quando o mtime do diretório
            # não muda (alguns FS/operacões).
            self.refresh_rows()
            return True

        def poll_registry_refresh(self) -> bool:
            # se o app estiver fechado, GLib para de chamar
            self.load_registry_async()
            return True

        def run_bg(self, work_fn, done_fn) -> None:
            def runner() -> None:
                result = work_fn()
                GLib.idle_add(done_fn, result)

            threading.Thread(target=runner, daemon=True).start()

    win = App()
    apply_css()
    win.show_all()
    Gtk.main()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
