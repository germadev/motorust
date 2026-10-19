"""Draws a centre line as a turtle: straights and arcs of given radius, in
the game's headings (0 north, -Z; turning left adds)."""
import math
import numpy as np

def fwd(h):
    return np.array([-math.sin(h), -math.cos(h)])

class Turtle:
    def __init__(self, x, z, heading, every=30.0):
        self.p = np.array([x, z], dtype=float)
        self.h = heading
        self.points = [tuple(self.p)]
        self.every = every
        self.marks = {}
        self.sections = {}

    def section(self, text):
        """A comment for the control points from here on."""
        self.sections[len(self.points)] = text
        return self

    def mark(self, name):
        self.marks[name] = (tuple(np.round(self.p, 2)), self.h)
        return self

    def emit(self):
        self.points.append(tuple(np.round(self.p, 2)))

    def S(self, length):
        n = max(1, int(round(length / (2 * self.every))))
        for _ in range(n):
            self.p = self.p + fwd(self.h) * (length / n)
            self.emit()
        return self

    def arc(self, radius, degrees, left):
        a = math.radians(degrees)
        n = max(1, int(round(radius * a / self.every)))
        sign = 1 if left else -1
        # centre to the left (or right) of the heading
        left_normal = np.array([fwd(self.h)[1], -fwd(self.h)[0]])
        centre = self.p + left_normal * radius * sign
        for k in range(1, n + 1):
            h = self.h + sign * a * k / n
            ln = np.array([fwd(h)[1], -fwd(h)[0]])
            q = centre - ln * radius * sign
            self.p = q
            self.emit()
        self.h += sign * a
        return self

    def L(self, radius, degrees):
        return self.arc(radius, degrees, True)

    def R(self, radius, degrees):
        return self.arc(radius, degrees, False)

    def jump(self, points, exit_point, heading):
        """Insert fixed points (a split's), then carry on from its exit."""
        for p in points:
            self.points.append(tuple(p))
        self.p = np.array(exit_point, dtype=float)
        self.h = heading
        return self

    def esses(self, length, amplitude_deg, bends, step=None):
        """A sine-generated curve, as a river meanders: the heading swings
        `amplitude_deg` either side and back, `bends` half-waves in
        `length` metres."""
        step = step or self.every / 2
        n = int(round(length / step))
        h0 = self.h
        amp = math.radians(amplitude_deg)
        for k in range(1, n + 1):
            s = k / n
            h = h0 + amp * math.sin(math.pi * bends * s)
            self.p = self.p + fwd(h) * (length / n)
            if k % 2 == 0 or k == n:
                self.emit()
        self.h = h0 + amp * math.sin(math.pi * bends)
        return self
