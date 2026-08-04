#!/bin/sh
# build.sh — clone, patch and build the three LLVM tools that can read Xbox 360
# PPC COFF. Optional: everything in tools/llvm/ works against a STOCK distro
# LLVM too (see README.md); this build only removes the scratch-copy step and
# adds relocation type names.
#
# Deliberately outside the repo: the source tree is 2.6 GB and the build tree
# 177 MB, and neither belongs in a git worktree. Set C2RS_LLVM_SRC to choose
# where it lands.
#
# Measured on this box (AMD Ryzen 9 7950X, 16 cores, gcc 16.1.1, ninja 1.13.2):
#   clone   --depth 1 --branch llvmorg-22.1.8    2.6 GB working tree, 291 MB .git
#   cmake   configure                            ~30 s
#   ninja -j8 llvm-readobj llvm-objdump llvm-mc  190 s wall, 870 targets
#   build tree                                   177 MB
#
# -j is 8 on purpose, not nproc: this box runs several lanes at once and an
# LLVM link is the memory spike. LLVM_PARALLEL_LINK_JOBS=1 keeps it to one.
set -eu

SRC="${C2RS_LLVM_SRC:-$HOME/build/llvm-w-llvm}"
TAG="${C2RS_LLVM_TAG:-llvmorg-22.1.8}"
JOBS="${C2RS_LLVM_JOBS:-8}"
here="$(cd "$(dirname "$0")" && pwd)"

for t in git cmake ninja; do
    command -v "$t" >/dev/null 2>&1 || { echo "SKIP: $t absent — cannot build LLVM"; exit 0; }
done

if [ ! -d "$SRC/.git" ]; then
    echo "cloning $TAG into $SRC (shallow; ~2.6 GB)"
    git clone --depth 1 --branch "$TAG" https://github.com/llvm/llvm-project.git "$SRC"
fi

cd "$SRC"
if git apply --check "$here/ppcbe.patch" 2>/dev/null; then
    git apply "$here/ppcbe.patch"
    echo "applied ppcbe.patch"
elif git apply --reverse --check "$here/ppcbe.patch" 2>/dev/null; then
    echo "ppcbe.patch already applied"
else
    echo "FAIL: ppcbe.patch does not apply to $(git describe --tags --always) — it is"
    echo "      written against llvmorg-22.1.8 (ca7933e47d3a3451d81e72ac174dcb5aa28b59d1)."
    exit 1
fi

cmake -S llvm -B build-ppcbe -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_TARGETS_TO_BUILD=PowerPC \
    -DLLVM_ENABLE_PROJECTS="" \
    -DLLVM_ENABLE_ASSERTIONS=OFF \
    -DLLVM_INCLUDE_TESTS=OFF -DLLVM_INCLUDE_BENCHMARKS=OFF -DLLVM_INCLUDE_EXAMPLES=OFF \
    -DLLVM_ENABLE_ZSTD=OFF -DLLVM_ENABLE_LIBXML2=OFF -DLLVM_ENABLE_ZLIB=OFF \
    -DLLVM_ENABLE_LIBEDIT=OFF \
    -DLLVM_PARALLEL_LINK_JOBS=1 -DLLVM_PARALLEL_COMPILE_JOBS="$JOBS" \
    -DLLVM_OPTIMIZED_TABLEGEN=ON >/dev/null

nice -n 15 ninja -C build-ppcbe -j"$JOBS" llvm-readobj llvm-objdump llvm-mc

echo ""
echo "built. point the tools at it with:"
echo "  export C2RS_LLVM_BIN=$SRC/build-ppcbe/bin"
