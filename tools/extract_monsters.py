BOXES = {
 'kraken': (5,5,300,145), 'skelkraken': (340,5,505,145), 'abyssPirate': (600,5,762,145), 'voodoo': (812,5,1002,145),
 'ghostGalley': (12,180,305,296), 'livingChest': (378,190,505,300), 'stormHarpy': (562,190,702,300), 'cannonAngler': (832,195,1002,300),
 'fleshShip': (22,345,332,488), 'mimicChest': (372,345,502,486), 'clockStorm': (522,345,772,488), 'cannonEater': (798,345,1012,488),
 'gorgonQueen': (30,512,222,652), 'krakenBasilisk': (262,518,502,652), 'manticore': (518,522,768,656), 'orcCaptain': (778,512,1012,656),
 'wyvern': (22,768,258,982), 'bellConstruct': (272,772,502,982), 'turtleWorm': (518,776,768,982), 'trueMimic': (782,772,1020,980),
}
# Cuts the monsters out of assets/monster-vorlage.jpg (the user's template sheet), removes the dark grid
# background, scales them to game size as pixel art and packs public/monsters.png + public/monsters.json.
# Usage: python3 tools/extract_monsters.py   (needs numpy, pillow, opencv, scikit-learn)
import cv2, numpy as np, json, os
from PIL import Image
from sklearn.cluster import KMeans
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TUNE = {"ghostGalley":[9,2],"abyssPirate":[8,3],"livingChest":[8,3],"mimicChest":[8,3],"cannonAngler":[8,3],"kraken":[12,2],"cannonEater":[16,1],"krakenBasilisk":[11,2],"manticore":[11,2],"gorgonQueen":[11,2],"stormHarpy":[10,2],"bellConstruct":[14,1]}
POCKETS = {'kraken': 60, 'krakenBasilisk': 60, 'bellConstruct': 80, 'clockStorm': 60, 'gorgonQueen': 80, 'fleshShip': 60, 'wyvern': 80, 'trueMimic': 80}
TARGET = {
 'kraken': 112, 'skelkraken': 72, 'abyssPirate': 36, 'voodoo': 52, 'ghostGalley': 120, 'livingChest': 30, 'stormHarpy': 52, 'cannonAngler': 56,
 'fleshShip': 120, 'mimicChest': 30, 'clockStorm': 140, 'cannonEater': 64, 'gorgonQueen': 112, 'krakenBasilisk': 132, 'manticore': 140, 'orcCaptain': 100,
 'wyvern': 100, 'bellConstruct': 100, 'turtleWorm': 140, 'trueMimic': 100,
}
def cut(img, k, box):
    x0, y0, x1, y1 = box
    T, R = TUNE.get(k, [13, 2])
    c = img[y0:y1, x0:x1]; h, w, _ = c.shape
    ring = np.concatenate([c[:5].reshape(-1,3), c[-5:].reshape(-1,3), c[:,:5].reshape(-1,3), c[:,-5:].reshape(-1,3)])
    km = KMeans(8, n_init=3, random_state=0).fit(ring)
    counts = np.bincount(km.labels_, minlength=8)
    cent = km.cluster_centers_[counts > len(ring) * 0.025]
    d = np.min(np.linalg.norm(c[:,:,None,:] - cent[None,None,:,:], axis=3), axis=2)
    bgc = (d < T).astype(np.uint8)
    fg0 = cv2.morphologyEx(1 - bgc, cv2.MORPH_OPEN, np.ones((2,2),np.uint8))
    seal = cv2.dilate(fg0, np.ones((2*R+1,2*R+1),np.uint8))
    _, lab = cv2.connectedComponents((1 - seal).astype(np.uint8), connectivity=4)
    border = set(np.unique(np.concatenate([lab[0],lab[-1],lab[:,0],lab[:,-1]]))) - {0}
    flooded = np.isin(lab, list(border)).astype(np.uint8)
    band = cv2.dilate(flooded, np.ones((2*R+3,2*R+3),np.uint8)) & (1 - flooded)
    fg = ((1 - flooded) & ~(band.astype(bool) & bgc.astype(bool))).astype(np.uint8)
    if k in POCKETS:
        pocket = (bgc & fg).astype(np.uint8)
        pn, pl, pst, _ = cv2.connectedComponentsWithStats(pocket, 4)
        for i in range(1, pn):
            if pst[i, cv2.CC_STAT_AREA] > POCKETS[k]: fg[pl == i] = 0
        fg = cv2.morphologyEx(fg, cv2.MORPH_OPEN, np.ones((2,2),np.uint8))
    n, labs, st, _ = cv2.connectedComponentsWithStats(fg, 8)
    if n > 1:
        big = st[1:,cv2.CC_STAT_AREA].max()
        fg = np.isin(labs, [i for i in range(1,n) if st[i,cv2.CC_STAT_AREA] > max(30, big*0.01)]).astype(np.uint8)
    a = np.dstack([c, fg*255]).astype(np.uint8)
    ys, xs = np.where(fg)
    return Image.fromarray(a[ys.min():ys.max()+1, xs.min():xs.max()+1], 'RGBA')
def pixelize(img, tw):
    a = np.asarray(img).astype(np.float32) / 255
    th = max(1, round(img.height * tw / img.width))
    pm = a.copy(); pm[..., :3] *= pm[..., 3:4]
    small = np.asarray(Image.fromarray((pm * 255).astype(np.uint8), 'RGBA').resize((tw, th), Image.BOX)).astype(np.float32) / 255
    al = small[..., 3]
    rgb = np.where(al[..., None] > 0.01, small[..., :3] / np.maximum(al[..., None], 1e-3), 0)
    l = rgb.mean(2, keepdims=True)
    rgb = np.clip((rgb - l) * 1.18 + l, 0, 1)
    rgb = np.clip((rgb - 0.5) * 1.1 + 0.52, 0, 1)
    alpha = al > 0.42
    q = Image.fromarray((rgb * 255).astype(np.uint8), 'RGB').quantize(colors=32, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert('RGB')
    arr = np.dstack([np.asarray(q), alpha * 255]).astype(np.uint8)
    A = alpha
    edge = A & ~(np.roll(A, 1, 0) & np.roll(A, -1, 0) & np.roll(A, 1, 1) & np.roll(A, -1, 1))
    o = arr[..., :3].astype(np.float32); o[edge] = o[edge] * np.array([0.6, 0.62, 0.72]); arr[..., :3] = o.astype(np.uint8)
    return Image.fromarray(arr, 'RGBA')
if __name__ == '__main__':
    img = np.asarray(Image.open(os.path.join(ROOT, 'assets/monster-vorlage.jpg')).convert('RGB')).astype(np.float32)
    sprites = {k: pixelize(cut(img, k, b), TARGET[k]) for k, b in BOXES.items()}
    # shelf pack into 512 wide
    W = 512; x = y = 2; rowh = 0; rects = {}
    for k, s in sorted(sprites.items(), key=lambda kv: -kv[1].height):
        if x + s.width + 2 > W: x = 2; y += rowh + 2; rowh = 0
        rects[k] = [x, y, s.width, s.height]; x += s.width + 2; rowh = max(rowh, s.height)
    H = y + rowh + 2
    atlas = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    for k, s in sprites.items(): atlas.paste(s, tuple(rects[k][:2]))
    atlas.save(os.path.join(ROOT, 'public/monsters.png'))
    json.dump(rects, open(os.path.join(ROOT, 'public/monsters.json'), 'w'))
    print('monsters.png', W, H, len(rects))
