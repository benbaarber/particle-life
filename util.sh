export WORKDIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

function syncwgsl() {
  python $WORKDIR/wgpu/sync_wgsl_types.py $WORKDIR/wgpu/src/shaders/types.wgsl $@
}
