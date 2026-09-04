#!/usr/bin/env python3
"""CJK 字体子集化:25MB 全量霞鹜文楷 -> 常用字子集,降 fontdue 解析常驻.
用法: python tools/subset_font.py [level1.txt 路径]
产物: assets/fonts/LXGWWenKai-Medium.ttf(覆盖),原文件备份到 assets/fonts/full/.
语料 = 一级字表3500 + 仓库全部源码/文案 CJK + ASCII/全角符号,缺字即报错.
"""
import re
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SRC = REPO / "src"
ASSETS = REPO / "assets"
FONT = ASSETS / "fonts" / "LXGWWenKai-Medium.ttf"
BACKUP = ASSETS / "fonts" / "full" / "LXGWWenKai-Medium.ttf"

CJK_PAT = re.compile(r"[\u2e80-\u9fff\uf900-\ufaff\U00020000-\U0002fa1d]")


def repo_chars() -> set[str]:
    out: set[str] = set()
    for base in (SRC, ASSETS):
        for p in base.rglob("*"):
            if not p.is_file() or p.suffix in {".lock"}:
                continue
            try:
                text = p.read_text(encoding="utf-8")
            except (UnicodeDecodeError, OSError):
                continue
            out.update(CJK_PAT.findall(text))
    return out


def main() -> None:
    from fontTools import subset

    level1 = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    chars: set[str] = set()
    if level1 is not None:
        chars.update(ch for ch in level1.read_text(encoding="utf-8").split() if ch.strip())
    used = repo_chars()
    chars.update(used)
    # ASCII 可打印 + 全角/ CJK 符号区 + 常用箭头/几何符号兜底
    chars.update(chr(c) for c in range(0x20, 0x7F))
    chars.update(chr(c) for c in list(range(0x3000, 0x3040)) + list(range(0xFF00, 0xFFEF)))
    chars.update("×÷★☆→←↑↓✓✗©®™…—‘’“”…·「」『』【】《》〈〉、。；：？！￥")
    unicodes = sorted({ord(c) for c in chars})
    ustr = ",".join(f"U+{u:04X}" for u in unicodes)
    print(f"corpus: level+repo={len(chars)} codepoints, repo-only CJK={len(used)}")

    if not BACKUP.exists():
        BACKUP.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(FONT, BACKUP)
        print(f"backup: {BACKUP} ({BACKUP.stat().st_size // 1024}KB)")
    before = FONT.stat().st_size

    opts = subset.Options()
    opts.name_IDs = ["*"]
    opts.name_legacy = True
    opts.name_languages = ["*"]
    opts.layout_features = ["*"]
    opts.no_hinting = True
    opts.desubroutinize = True
    opts.drop_tables += ["DSIG"]
    font = subset.load_font(str(FONT), opts)
    ss = subset.Subsetter(opts)
    ss.populate(unicodes=unicodes)
    ss.subset(font)
    subset.save_font(font, str(FONT), opts)
    after = FONT.stat().st_size
    print(f"subset: {before // 1024}KB -> {after // 1024}KB")

    # 覆盖校验:仓库每个字必须在子集 cmap 里
    from fontTools.ttLib import TTFont

    cmap = TTFont(str(FONT)).getBestCmap()
    missing = sorted({c for c in used if ord(c) not in cmap})
    if missing:
        print(f"MISSING {len(missing)}: {''.join(missing[:50])}")
        sys.exit(1)
    print("coverage: all repo CJK present")


if __name__ == "__main__":
    main()
