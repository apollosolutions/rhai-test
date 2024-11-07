#!/bin/bash
#
# Licensed under the MIT license
# <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# This is just a little script that can be downloaded from the internet to
# install rhai-test. It downloads the rhai-test tarball from GitHub releases.

# Example to bypass binary overwrite [y/N] prompt
# curl -sSL https://rhai-test.apollo.dev/nix/latest | sh -s -- --force

set -u

BINARY_DOWNLOAD_PREFIX="https://github.com/apollosolutions/rhai-test/releases/download"

# Binary version defined in root cargo.toml
# Note: this line is built automatically
# in build.rs. Don't touch it!
PACKAGE_VERSION="v0.2.4"

say() {
    local green=`tput setaf 2 2>/dev/null || echo ''`
    local reset=`tput sgr0 2>/dev/null || echo ''`
    echo "$1"
}

err() {
    local red=`tput setaf 1 2>/dev/null || echo ''`
    local reset=`tput sgr0 2>/dev/null || echo ''`
    say "${red}ERROR${reset}: $1" >&2
    exit 1
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
    return $?
}

need_cmd() {
    if ! check_cmd "$1"
    then err "need '$1' (command not found)"
    fi
}

# This wraps curl or wget. Try curl first, if not installed,
# use wget instead.
downloader() {
    if check_cmd curl
    then _dld=curl
    elif check_cmd wget
    then _dld=wget
    else _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]
    then need_cmd "$_dld"
    elif [ "$_dld" = curl ]
    then curl -sSfL "$1" -o "$2"
    elif [ "$_dld" = wget ]
    then wget "$1" -O "$2"
    else err "Unknown downloader"   # should not reach here
    fi
}

need_ok() {
    if [ $? != 0 ]; then err "$1"; fi
}

has_required_glibc() {
    local _ldd_version="$(ldd --version 2>&1 | head -n1)"
    # glibc version string is inconsistent across distributions
    # instead check if the string does not contain musl (case insensitive)
    if echo "${_ldd_version}" | grep -iv musl >/dev/null; then
        local _glibc_version=$(echo "${_ldd_version}" | awk 'NR==1 { print $NF }')
        local _glibc_major_version=$(echo "${_glibc_version}" | cut -d. -f1)
        local _glibc_min_version=$(echo "${_glibc_version}" | cut -d. -f2)
        local _min_major_version=2
        local _min_minor_version=35
        if [ "${_glibc_major_version}" -gt "${_min_major_version}" ] \
            || { [ "${_glibc_major_version}" -eq "${_min_major_version}" ] \
            && [ "${_glibc_min_version}" -ge "${_min_minor_version}" ]; }; then
            return 0
        else
            say "This operating system needs glibc >= ${_min_major_version}.${_min_minor_version}, but only has ${_libc_version} installed."
        fi
    else
        say "This operating system does not support dynamic linking to glibc."
    fi

    return 1
}

get_architecture() {
    local _ostype="$(uname -s)"
    local _cputype="$(uname -m)"

    if [ "$_ostype" = Darwin -a "$_cputype" = i386 ]; then
        # Darwin `uname -s` lies
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            local _cputype=x86_64
        fi
    fi

    if [ "$_ostype" = Darwin -a "$_cputype" = arm64 ]; then
        # Darwin `uname -s` doesn't seem to lie on Big Sur
        # but the cputype we want is called aarch64, not arm64 (they are equivalent)
        local _cputype=aarch64
    fi

    case "$_ostype" in
        Linux)
            if has_required_glibc; then
                local _ostype=unknown-linux-gnu
            else
                err "glibc library version was not sufficiant and musl is not supported. Please upgrade your glibc version."
            fi
            ;;

        Darwin)
            local _ostype=apple-darwin
            ;;

        MINGW* | MSYS* | CYGWIN*)
            local _ostype=pc-windows-msvc
            ;;

        *)
            err "no precompiled binaries available for OS: $_ostype"
            ;;
    esac

    case "$_cputype" in
        # these are the only two acceptable values for cputype
        x86_64 | aarch64 )
            ;;
        *)
            err "no precompiled binaries available for CPU architecture: $_cputype"

    esac

    local _arch="$_cputype-$_ostype"

    RETVAL="$_arch"
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    "$@"
    need_ok "command failed: $*"
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    "$@"
}

get_home_dir() {
  if [ -n "$HOME" ]; then
    echo "$HOME"
  elif [ "$(uname)" = "Darwin" ] || [ "$(uname)" = "Linux" ]; then
    echo "Error: Home directory not found on Unix-like system"
    return 1
  elif [ "$(uname -o)" = "Cygwin" ] || [ "$(uname -o)" = "Msys" ]; then
    echo "Error: Home directory not found on Windows system"
    return 1
  else
    echo "Error: Unsupported operating system"
    return 1
  fi
}

download_binary_and_run_installer() {
    downloader --check
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd rmdir
    need_cmd tar
    need_cmd which
    need_cmd dirname
    need_cmd awk
    need_cmd cut

    # if $VERSION isn't provided or has 0 length, use version from Rhai-test cargo.toml
    # ${VERSION:-} checks if version exists, and if doesn't uses the default
    # which is after the :-, which in this case is empty. -z checks for empty str
    if [ -z ${VERSION:-} ]; then
        # VERSION is either not set or empty
        DOWNLOAD_VERSION=$PACKAGE_VERSION
    else
        # VERSION set and not empty
        DOWNLOAD_VERSION=$VERSION
    fi

    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    local _ext=""
    case "$_arch" in
        *windows*)
            _ext=".exe"
            ;;
    esac

    local _tardir="rhai-test-${_arch}"
    local _url="$BINARY_DOWNLOAD_PREFIX/$DOWNLOAD_VERSION/${_tardir}.tgz"
    local _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t rhai-test)"
    local _file="$_dir/input.tgz"

    say "downloading rhai-test from $_url" 1>&2

    ensure mkdir -p "$_dir"
    downloader "$_url" "$_file"
    if [ $? != 0 ]; then
      say "failed to download $_url"
      say "this may be a standard network error, but it may also indicate"
      say "that rhai-test's release process is not working. When in doubt"
      say "please feel free to open an issue!"
      say "https://github.com/apollosolutions/rhai-test/issues/new/choose"
      exit 1
    fi

    ensure tar xf "$_file" --strip-components 1 -C "$_dir"

    local _retval=$?

    local _home_dir=$(get_home_dir)
    local _bin_folder="$_home_dir/.rhai-test/bin"

    mkdir -p "$_bin_folder"

    echo "Copying file to bin directory..."

    cp "$_dir/rhai-test" "$_bin_folder/rhai-test"

    echo "Adding to path..."

    # Add bin folder to path
    # Check if _bin_folder is already in PATH
    if [[ ":$PATH:" != *":$_bin_folder:"* && "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]] || \
        [[ ";$PATH;" != *";$_bin_folder;"* && ("$OSTYPE" == "msys" || "$OSTYPE" == "cygwin") ]]; then
        # Add _bin_folder to PATH
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
            # For Windows (Git Bash/Cygwin)
            export PATH="$PATH;$_bin_folder"
            echo "export PATH=\"\$PATH;$_bin_folder\"" >> "$HOME/.bashrc"
        else
            # For Unix-like systems
            export PATH="$PATH:$_bin_folder"
            echo "export PATH=\"\$PATH:$_bin_folder\"" >> "$HOME/.bashrc"
            echo "export PATH=\"\$PATH:$_bin_folder\"" >> "$HOME/.zshrc"
        fi
    fi

    ignore rm -rf "$_dir"

    echo "rhai-test installed successfully! You can now run rhai-test in your terminal."
    echo "Please review the README for how to use this tool: https://github.com/apollosolutions/rhai-test"

    return "$_retval"
}

download_binary_and_run_installer "$@" || exit 1