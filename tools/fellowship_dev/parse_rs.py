import re
def array(src, name):
    m = re.search(r"const " + name + r": \[[^\]]+\] = \[(.*?)\n\];", src, re.S)
    body = re.sub(r"//[^\n]*", "", m.group(1))
    if "Vec2::new" in body:
        return [(float(a), float(b)) for a, b in re.findall(r"Vec2::new\(\s*(-?[\d.]+),\s*(-?[\d.]+)\)", body)]
    return [(float(a), float(b)) for a, b in re.findall(r"\(\s*(-?[\d.]+),\s*(-?[\d.]+)\)", body)]
