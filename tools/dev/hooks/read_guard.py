#!/usr/bin/env python3
"""PreToolUse-Hook für Read (eingetragen in .claude/settings.json): verhindert, dass große Textdateien
versehentlich ganz gelesen werden – der teuerste Fehler in diesem Projekt (world/mod.rs: 4100 Zeilen).

Erlaubt bleibt alles Kleine (bis 600 Zeilen und 40 kB), jedes Read mit `limit` (bewusst gewählt),
Bilder, PDFs und Notizbücher. Bei jedem Fehler lässt der Hook das Lesen zu.
"""
import json
import os
import sys

MAX_LINES, MAX_BYTES = 600, 40_000
BINARY = ('.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.ico', '.svg', '.pdf', '.ipynb')


def main():
    d = json.load(sys.stdin)
    ti = d.get('tool_input') or {}
    p = ti.get('file_path') or ''
    if d.get('tool_name') != 'Read' or not p or ti.get('limit') or ti.get('pages') or ti.get('allow_large'):
        return
    if p.lower().endswith(BINARY) or not os.path.isfile(p):
        return
    size = os.path.getsize(p)
    if size <= MAX_BYTES and size < MAX_LINES * 20:
        return
    with open(p, 'rb') as f:
        lines = f.read().count(b'\n') + 1
    rest = lines - int(ti.get('offset') or 0)
    if (rest <= MAX_LINES and size <= MAX_BYTES) or (ti.get('offset') and rest <= MAX_LINES):
        return
    rel = os.path.relpath(p, os.environ.get('CLAUDE_PROJECT_DIR') or os.getcwd())
    if rel.startswith('..'):
        rel = p
    if p.endswith(('.rs', '.wgsl')):
        tip = f'`tools/dev/code ol {rel}` (Gliederung), `tools/dev/code fn <name>` (ein Element), `tools/dev/g <muster>` (Suche)'
    elif p.endswith('.md'):
        tip = '`tools/dev/doc <stichwort>` (Suche mit Abschnitt), `tools/dev/doc -s <überschrift>` (ein Abschnitt)'
    else:
        tip = f'`rg -n -M 180 <muster> {rel}` (Suche)'
    why = (f'Große Datei ({lines} Zeilen, {size // 1000} kB) – nicht ganz lesen. Billiger: {tip} '
           f'oder Read mit offset und limit. Wer wirklich alles braucht, gibt limit an.')
    print(json.dumps({'hookSpecificOutput': {'hookEventName': 'PreToolUse', 'permissionDecision': 'deny',
                                             'permissionDecisionReason': why}}, ensure_ascii=False))


if __name__ == '__main__':
    try:
        main()
    except Exception:
        pass
    sys.exit(0)
