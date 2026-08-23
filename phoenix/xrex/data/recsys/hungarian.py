# SPDX-License-Identifier: Apache-2.0
"""Kuhn–Munkres (Hungarian) assignment. Maximize profit, O(n³)."""

from __future__ import annotations

import numpy as np

INF = 1e100


def hungarian_minimize(cost: np.ndarray) -> np.ndarray:
    """Return col_of_row for a square cost matrix (minimize)."""
    a = np.asarray(cost, dtype=np.float64)
    n = a.shape[0]
    if a.shape != (n, n):
        raise ValueError("cost must be square")
    u = np.zeros(n + 1)
    v = np.zeros(n + 1)
    p = np.zeros(n + 1, dtype=np.int64)
    way = np.zeros(n + 1, dtype=np.int64)
    for i in range(1, n + 1):
        p[0] = i
        j0 = 0
        minv = np.full(n + 1, INF)
        used = np.zeros(n + 1, dtype=bool)
        while True:
            used[j0] = True
            i0 = int(p[j0])
            delta = INF
            j1 = 0
            for j in range(1, n + 1):
                if used[j]:
                    continue
                cur = a[i0 - 1, j - 1] - u[i0] - v[j]
                if cur < minv[j]:
                    minv[j] = cur
                    way[j] = j0
                if minv[j] < delta:
                    delta = minv[j]
                    j1 = j
            for j in range(0, n + 1):
                if used[j]:
                    u[int(p[j])] += delta
                    v[j] -= delta
                else:
                    minv[j] -= delta
            j0 = j1
            if p[j0] == 0:
                break
        while True:
            j1 = int(way[j0])
            p[j0] = p[j1]
            j0 = j1
            if j0 == 0:
                break
    col_of_row = np.zeros(n, dtype=np.int64)
    for j in range(1, n + 1):
        if p[j] > 0:
            col_of_row[int(p[j]) - 1] = j - 1
    return col_of_row


def hungarian_maximize(profit: np.ndarray) -> np.ndarray:
    p = np.asarray(profit, dtype=np.float64)
    return hungarian_minimize(p.max() - p)


def assign_slots(profit: np.ndarray) -> list[tuple[int, int]]:
    """Rows = posts, cols = slots. Returns (post_index, slot_index) for real slots."""
    p = np.asarray(profit, dtype=np.float64)
    rows, cols = p.shape
    n = max(rows, cols)
    square = np.zeros((n, n), dtype=np.float64)
    square[:rows, :cols] = p
    col_of_row = hungarian_maximize(square)
    out = []
    for i in range(rows):
        j = int(col_of_row[i])
        if j < cols:
            out.append((i, j))
    out.sort(key=lambda x: x[1])
    return out
