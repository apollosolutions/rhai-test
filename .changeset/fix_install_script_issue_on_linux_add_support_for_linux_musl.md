---
default: minor
---

# Fix install script issue on linux, add support for linux-musl

Fixed issue in install script that would result in "has_required_glibc: command not found" error and then it attempting to install linux-musl which did not exist.
