#!/bin/bash

SCRIPT_REPO="https://github.com/xiph/ogg.git"
SCRIPT_TAG="v1.3.5"

ffbuild_dockerbuild() {
  git-mini-clone "$SCRIPT_REPO" "$SCRIPT_TAG" ogg
  cd ogg

  ./autogen.sh

  local myconf=(
    --prefix="$FFBUILD_PREFIX"
    --disable-shared
    --enable-static
    --with-pic
  )

  if [[ $TARGET == win* || $TARGET == linux* ]]; then
    myconf+=(
      --host="$FFBUILD_TOOLCHAIN"
    )
  else
    echo "Unknown target"
    return 255
  fi

  ./configure "${myconf[@]}"
  make -j"$(nproc)"
  make install
}
