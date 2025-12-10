#!/usr/bin/env python
# -*- coding: utf-8 -*-
#
# @Author: José Sánchez-Gallego (gallegoj@uw.edu)
# @Date: 2025-12-10
# @Filename: _rheader.pyi
# @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)

class Header:
    """Representation of a header."""

    keywords: dict[str, Keyword]

class Keyword:
    """Representation of a header keyword."""

    name: str
    value: str | int | float | bool | None
    comment: str | None

def read_header(file_path: str) -> dict:
    """Reads a header from a file and returns it as a dictionary."""
    ...

def read_header_to_class(file_path: str, cls: type) -> Header:
    """Reads a header from a file and returns it as an instance of :obj:`.Header`."""
    ...
