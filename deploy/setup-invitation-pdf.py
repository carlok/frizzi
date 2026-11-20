from concurrent.futures import ThreadPoolExecutor
from hashlib import sha256
from pathlib import Path
from platform import machine
from tempfile import NamedTemporaryFile
from urllib.parse import quote
from urllib.request import urlopen
from argparse import ArgumentParser
import tarfile
from io import BytesIO


def download(url):
    with urlopen(url, timeout=60) as response:
        return response.read()


def install(path, content, mode=0o644):
    path.parent.mkdir(parents=True, exist_ok=True)
    with NamedTemporaryFile(dir=path.parent, delete=False) as temporary:
        temporary.write(content)
        staged = Path(temporary.name)
    try:
        staged.chmod(mode)
        staged.replace(path)
    finally:
        staged.unlink(missing_ok=True)


if machine() != "x86_64":
    raise SystemExit("Install Typst manually on this architecture.")

parser = ArgumentParser()
parser.add_argument("--bundle", type=Path, help="Install into a self-contained deployment directory.")
bundle = parser.parse_args().bundle
binary = bundle / "typst" if bundle else Path.home() / ".local/bin/typst"
font_dir = bundle / "fonts" if bundle else Path.home() / ".local/share/frizzi/pdf-fonts"
release = "typst-x86_64-unknown-linux-musl"
archive_hash = "a6d077d0a95eed5a2eba715b2dae06be954f624ccbf85758a03f389ded33118c"
fonts_source = "https://raw.githubusercontent.com/google/fonts/cc5661d43cd4f9aad5567033a8e15ca28dc35c86/ofl/"
fonts = {
    "spacegrotesk": "SpaceGrotesk[wght].ttf",
    "notosansarabic": "NotoSansArabic[wdth,wght].ttf",
    "notosansbengali": "NotoSansBengali[wdth,wght].ttf",
    "notosansdevanagari": "NotoSansDevanagari[wdth,wght].ttf",
    "notosanssc": "NotoSansSC[wght].ttf",
    "notoemoji": "NotoEmoji[wght].ttf",
}


def install_font(item):
    family, file = item
    for asset in (file, "OFL.txt"):
        relative = f"{family}/{asset}"
        install(font_dir / relative, download(fonts_source + quote(relative)))


with ThreadPoolExecutor() as pool:
    pending = [pool.submit(install_font, item) for item in fonts.items()]
    archive = download(f"https://github.com/typst/typst/releases/download/v0.15.1/{release}.tar.xz")
    if sha256(archive).hexdigest() != archive_hash:
        raise SystemExit("Typst archive checksum mismatch.")
    with tarfile.open(fileobj=BytesIO(archive), mode="r:xz") as package:
        file = package.extractfile(f"{release}/typst")
        if file is None:
            raise SystemExit("Typst executable missing from the archive.")
        install(binary, file.read(), 0o755)
    for task in pending:
        task.result()
