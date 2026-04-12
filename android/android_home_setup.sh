# $ source android_home_setup.sh
# Taken from https://wiki.archlinux.org/title/Android#Using_/opt/android-sdk_as_read-only_with_CoW
LOWER=/opt/android-sdk
UPPER="$HOME/.local/android/.sdk/upper"
WORK="$HOME/.local/android/.sdk/work"
ANDROID_HOME="$HOME/.local/android/sdk"
mkdir -p "$UPPER" "$WORK" "$ANDROID_HOME"
fuse-overlayfs -o squash_to_uid=$(id -u),squash_to_gid=$(id -g),lowerdir=$LOWER,upperdir=$UPPER,workdir=$WORK $ANDROID_HOME
export ANDROID_HOME
