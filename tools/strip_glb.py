#!/usr/bin/env python3
"""Shrink a KayKit .glb: keep only the named animations and repack the binary buffer.
usage: strip_glb.py in.glb out.glb Anim1,Anim2,...  (no third-party packages needed)"""
import json, struct, sys

def read_glb(path):
    b = open(path, 'rb').read()
    assert b[:4] == b'glTF'
    off, j, bin_ = 12, None, b''
    while off < len(b):
        ln, typ = struct.unpack('<II', b[off:off + 8])
        chunk = b[off + 8:off + 8 + ln]
        if typ == 0x4E4F534A: j = json.loads(chunk)
        elif typ == 0x004E4942: bin_ = chunk
        off += 8 + ln
    return j, bin_

def write_glb(path, j, bin_):
    js = json.dumps(j, separators=(',', ':')).encode()
    js += b' ' * ((4 - len(js) % 4) % 4)
    bin_ += b'\0' * ((4 - len(bin_) % 4) % 4)
    total = 12 + 8 + len(js) + 8 + len(bin_)
    with open(path, 'wb') as f:
        f.write(struct.pack('<III', 0x46546C67, 2, total))
        f.write(struct.pack('<II', len(js), 0x4E4F534A)); f.write(js)
        f.write(struct.pack('<II', len(bin_), 0x004E4942)); f.write(bin_)

def main():
    src, dst, keep = sys.argv[1], sys.argv[2], set(sys.argv[3].split(','))
    j, bin_ = read_glb(src)
    j['animations'] = [a for a in j.get('animations', []) if a['name'] in keep]
    used_acc = set()
    for m in j.get('meshes', []):
        for p in m['primitives']:
            used_acc.update(p['attributes'].values())
            if 'indices' in p: used_acc.add(p['indices'])
            for t in p.get('targets', []): used_acc.update(t.values())
    for s in j.get('skins', []):
        if 'inverseBindMatrices' in s: used_acc.add(s['inverseBindMatrices'])
    for a in j['animations']:
        for s in a['samplers']: used_acc.update([s['input'], s['output']])
    acc_map = {}
    new_acc = []
    for i, a in enumerate(j['accessors']):
        if i in used_acc: acc_map[i] = len(new_acc); new_acc.append(a)
    used_bv = {a['bufferView'] for a in new_acc if 'bufferView' in a}
    used_bv |= {im['bufferView'] for im in j.get('images', []) if 'bufferView' in im}
    bv_map, new_bv, out = {}, [], bytearray()
    for i, bv in enumerate(j['bufferViews']):
        if i not in used_bv: continue
        while len(out) % 4: out.append(0)
        start = bv.get('byteOffset', 0)
        data = bin_[start:start + bv['byteLength']]
        nb = dict(bv); nb['byteOffset'] = len(out); nb['buffer'] = 0
        out += data
        bv_map[i] = len(new_bv); new_bv.append(nb)
    for a in new_acc:
        if 'bufferView' in a: a['bufferView'] = bv_map[a['bufferView']]
    for im in j.get('images', []):
        if 'bufferView' in im: im['bufferView'] = bv_map[im['bufferView']]
    for m in j.get('meshes', []):
        for p in m['primitives']:
            p['attributes'] = {k: acc_map[v] for k, v in p['attributes'].items()}
            if 'indices' in p: p['indices'] = acc_map[p['indices']]
            if 'targets' in p: p['targets'] = [{k: acc_map[v] for k, v in t.items()} for t in p['targets']]
    for s in j.get('skins', []):
        if 'inverseBindMatrices' in s: s['inverseBindMatrices'] = acc_map[s['inverseBindMatrices']]
    for a in j['animations']:
        for s in a['samplers']: s['input'] = acc_map[s['input']]; s['output'] = acc_map[s['output']]
    j['accessors'] = new_acc; j['bufferViews'] = new_bv
    j['buffers'] = [{'byteLength': len(out)}]
    write_glb(dst, j, bytes(out))
    print(dst, len(j['animations']), 'animations,', round(len(out) / 1024), 'KB')

main()
