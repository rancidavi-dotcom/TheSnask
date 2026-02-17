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

def run_cmd(args: List[str], cwd: Optional[str] = None, timeout_s: int = 60 * 15) -> Tuple[int, str]:
    try:
        p = subprocess.run(
            args,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout_s,
        )
        return p.returncode, (p.stdout or "")
    except subprocess.TimeoutExpired:
        return 124, "timeout"
    except FileNotFoundError:
        return 127, f"command not found: {args[0]}"
    except Exception as e:
        return 1, str(e)


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
            self.refresh_btn.set_tooltip_text("Atualizar tudo (Snask + bibliotecas + registry)")
            self.refresh_btn.connect("clicked", self.on_update_all)
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

            # Main: tabs
            notebook = Gtk.Notebook()
            root.pack_start(notebook, True, True, 0)

            # Tab 1: Pacotes (scroll feed - Play Store-like)
            packages_tab = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
            notebook.append_page(packages_tab, Gtk.Label(label="Pacotes"))

            sc = Gtk.ScrolledWindow()
            sc.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)
            packages_tab.pack_start(sc, True, True, 0)

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

            # Tab 2: Dev (criar/publicar libs)
            dev_tab = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
            dev_tab.set_border_width(12)
            notebook.append_page(dev_tab, Gtk.Label(label="Dev"))

            dev_title = Gtk.Label(label="Criar e publicar bibliotecas Snask")
            dev_title.set_xalign(0.0)
            dev_title.get_style_context().add_class("snask-hero")
            dev_tab.pack_start(dev_title, False, False, 0)

            form = Gtk.Grid()
            form.set_row_spacing(8)
            form.set_column_spacing(10)
            dev_tab.pack_start(form, False, False, 0)

            lbl_dir = Gtk.Label(label="Diretório:")
            lbl_dir.set_xalign(0.0)
            form.attach(lbl_dir, 0, 0, 1, 1)
            self.dev_dir = Gtk.Entry()
            self.dev_dir.set_text(os.path.join(home_dir(), "SnaskLibs"))
            form.attach(self.dev_dir, 1, 0, 1, 1)
            btn_pick = Gtk.Button(label="Escolher…")
            form.attach(btn_pick, 2, 0, 1, 1)

            def pick_dir(_w) -> None:
                dlg = Gtk.FileChooserDialog(
                    title="Selecione um diretório",
                    parent=self,
                    action=Gtk.FileChooserAction.SELECT_FOLDER,
                    buttons=(Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL, Gtk.STOCK_OPEN, Gtk.ResponseType.OK),
                )
                dlg.set_current_folder(self.dev_dir.get_text() or home_dir())
                resp = dlg.run()
                if resp == Gtk.ResponseType.OK:
                    self.dev_dir.set_text(dlg.get_filename() or "")
                dlg.destroy()

            btn_pick.connect("clicked", pick_dir)

            lbl_name = Gtk.Label(label="Nome:")
            lbl_name.set_xalign(0.0)
            form.attach(lbl_name, 0, 1, 1, 1)
            self.dev_name = Gtk.Entry()
            self.dev_name.set_placeholder_text("minha_lib")
            form.attach(self.dev_name, 1, 1, 2, 1)

            lbl_ver = Gtk.Label(label="Versão:")
            lbl_ver.set_xalign(0.0)
            form.attach(lbl_ver, 0, 2, 1, 1)
            self.dev_ver = Gtk.Entry()
            self.dev_ver.set_text("0.1.0")
            form.attach(self.dev_ver, 1, 2, 2, 1)

            lbl_desc = Gtk.Label(label="Descrição:")
            lbl_desc.set_xalign(0.0)
            form.attach(lbl_desc, 0, 3, 1, 1)
            self.dev_desc = Gtk.Entry()
            self.dev_desc.set_placeholder_text("Uma biblioteca Snask…")
            form.attach(self.dev_desc, 1, 3, 2, 1)

            actions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            dev_tab.pack_start(actions, False, False, 0)

            self.dev_btn_create = Gtk.Button(label="Criar template")
            self.dev_btn_publish = Gtk.Button(label="Publicar no GitHub")
            self.dev_push = Gtk.CheckButton(label="git push automaticamente")
            self.dev_push.set_active(True)
            self.dev_pr = Gtk.CheckButton(label="Enviar como PR (fork)")
            self.dev_fork = Gtk.Entry()
            self.dev_fork.set_placeholder_text("https://github.com/SEUUSER/SnaskPackages")
            actions.pack_start(self.dev_btn_create, False, False, 0)
            actions.pack_start(self.dev_btn_publish, False, False, 0)
            actions.pack_start(self.dev_push, False, False, 0)
            actions.pack_start(self.dev_pr, False, False, 0)

            fork_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            dev_tab.pack_start(fork_row, False, False, 0)
            fork_lbl = Gtk.Label(label="Fork URL (para PR):")
            fork_lbl.set_xalign(0.0)
            fork_row.pack_start(fork_lbl, False, False, 0)
            fork_row.pack_start(self.dev_fork, True, True, 0)

            self.dev_log = Gtk.TextView()
            self.dev_log.set_editable(False)
            self.dev_log.set_cursor_visible(False)
            self.dev_log.get_style_context().add_class("snask-subtle")
            dev_sc = Gtk.ScrolledWindow()
            dev_sc.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)
            dev_sc.set_size_request(-1, 220)
            dev_sc.add(self.dev_log)
            dev_tab.pack_start(dev_sc, True, True, 0)

            def log_append(text: str) -> None:
                buf = self.dev_log.get_buffer()
                end = buf.get_end_iter()
                buf.insert(end, text)

            def dev_validate() -> Tuple[str, str, str, str]:
                d = (self.dev_dir.get_text() or "").strip()
                n = (self.dev_name.get_text() or "").strip()
                v = (self.dev_ver.get_text() or "").strip()
                ds = (self.dev_desc.get_text() or "").strip()
                return d, n, v, ds

            def dev_create(_w) -> None:
                d, n, v, ds = dev_validate()
                if not d or not n:
                    self.status_lbl.set_text("Dev: preencha diretório e nome.")
                    return
                os.makedirs(d, exist_ok=True)
                self.status_lbl.set_text(f"Dev: criando {n}…")
                self.spinner.start()
                self.dev_btn_create.set_sensitive(False)
                self.dev_btn_publish.set_sensitive(False)
                log_append(f"$ snask lib init {n} --version {v} --description \"{ds}\"\n")

                def work():
                    return run_cmd(["snask", "lib", "init", n, "--version", v, "--description", ds], cwd=d)

                def done(result):
                    code, out = result
                    log_append(out + ("" if out.endswith("\n") else "\n"))
                    self.spinner.stop()
                    self.dev_btn_create.set_sensitive(True)
                    self.dev_btn_publish.set_sensitive(True)
                    if code == 0:
                        self.status_lbl.set_text(f"Dev: template criado ({n}.snask).")
                    else:
                        self.status_lbl.set_text(f"Dev: erro ao criar (code={code}).")

                self.run_bg(work, done)

            def dev_publish(_w) -> None:
                d, n, v, ds = dev_validate()
                if not d or not n:
                    self.status_lbl.set_text("Dev: preencha diretório e nome.")
                    return
                if not os.path.exists(os.path.join(d, f"{n}.snask")):
                    self.status_lbl.set_text(f"Dev: não achei {n}.snask em {d}.")
                    return
                self.status_lbl.set_text(f"Dev: publicando {n}…")
                self.spinner.start()
                self.dev_btn_create.set_sensitive(False)
                self.dev_btn_publish.set_sensitive(False)
                args = ["snask", "lib", "publish", n, "--version", v, "--description", ds]
                if self.dev_pr.get_active():
                    fork = (self.dev_fork.get_text() or "").strip()
                    if not fork:
                        self.spinner.stop()
                        self.dev_btn_create.set_sensitive(True)
                        self.dev_btn_publish.set_sensitive(True)
                        self.status_lbl.set_text("Dev: informe o Fork URL para PR.")
                        return
                    args += ["--pr", "--fork", fork]
                elif self.dev_push.get_active():
                    args.append("--push")
                log_append("$ " + " ".join(args) + "\n")

                def work():
                    return run_cmd(args, cwd=d)

                def done(result):
                    code, out = result
                    log_append(out + ("" if out.endswith("\n") else "\n"))
                    self.spinner.stop()
                    self.dev_btn_create.set_sensitive(True)
                    self.dev_btn_publish.set_sensitive(True)
                    if code == 0:
                        self.status_lbl.set_text(f"Dev: publicado {n}.")
                        self.load_registry_async()
                    else:
                        self.status_lbl.set_text(f"Dev: erro ao publicar (code={code}).")

                self.run_bg(work, done)

            self.dev_btn_create.connect("clicked", dev_create)
            self.dev_btn_publish.connect("clicked", dev_publish)

            self.cards: List[PkgCard] = []

            self.load_registry_async()
            # Mantém a UI sempre consistente: se algo mexer em ~/.snask/packages (CLI ou GUI),
            # atualiza os badges e botões automaticamente.
            GLib.timeout_add_seconds(2, self.poll_installed_state)
            # Atualiza registry periodicamente (mantém catálogo em dia).
            GLib.timeout_add_seconds(120, self.poll_registry_refresh)

        def on_search_changed(self, _w) -> None:
            self.apply_filter()

        def on_update_all(self, _w) -> None:
            # Atualiza o Snask (binário) e depois atualiza todas as libs instaladas.
            self.status_lbl.set_text("Atualizando Snask + bibliotecas…")
            self.refresh_btn.set_sensitive(False)
            self.spinner.start()

            def work():
                logs = []
                print("[store] update: snask update (sistema)…")
                code, out = run_cmd(["snask", "update"])
                logs.append(("snask update", code, out))

                installed = []
                try:
                    ensure_packages_dir()
                    for fn in os.listdir(packages_dir()):
                        if fn.endswith(".snask"):
                            installed.append(os.path.splitext(fn)[0])
                except Exception:
                    installed = []

                for name in sorted(set(installed)):
                    print(f"[store] update: snask update {name}…")
                    c2, o2 = run_cmd(["snask", "update", name])
                    logs.append((f"snask update {name}", c2, o2))
                return logs

            def done(logs):
                self.refresh_btn.set_sensitive(True)
                self.spinner.stop()
                failed = [cmd for (cmd, code, _out) in logs if code != 0]
                if failed:
                    self.status_lbl.set_text(f"Atualização concluída com falhas: {len(failed)} (veja terminal).")
                else:
                    self.status_lbl.set_text("Atualização concluída (Snask + bibliotecas).")
                self.refresh_rows()
                self.load_registry_async()

            self.run_bg(work, done)

        def load_registry_async(self) -> None:
            self.status_lbl.set_text("Sincronizando registry…")
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
