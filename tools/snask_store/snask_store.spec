# PyInstaller spec for Snask Store

# pylint: disable=all

from PyInstaller.utils.hooks import collect_data_files

block_cipher = None

datas = [
    ("assets", "assets"),
]

a = Analysis(
    ["snask_store.py"],
    pathex=["tools/snask_store"],
    binaries=[],
    datas=datas,
    hiddenimports=["gi"],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="snask-store",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
)
