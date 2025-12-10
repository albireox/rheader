#!/usr/bin/env python
# -*- coding: utf-8 -*-
#
# @Author: José Sánchez-Gallego (gallegoj@uw.edu)
# @Date: 2025-12-09
# @Filename: __init__.py
# @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)

from __future__ import annotations

__all__ = [
    "Header",
    "Keyword",
    "read_header",
    "read_header_to_class",
]


from ._rheader import Header, Keyword, read_header, read_header_to_class
