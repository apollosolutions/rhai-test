---
default: minor
---

# Fix install script issue on linux

Fixed issue in install script that would result in "has_required_glibc: command not found" error and then it attempting to install linux-musl which does not exist.
