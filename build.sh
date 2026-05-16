#!/usr/bin/env bash

# build rust crate for various targets
set -umeTE
set +f
set -o pipefail
export IFS=$'\n'

declare -- script_name="$(basename "${BASH_SOURCE[0]}")"
declare -- script_path="$(2>/dev/random 1>/dev/random cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
declare -- this_script_path="${script_path}/${script_name}"
declare -- stderr="$(mktemp)"

on_exit() {
    set +x
    bash -c "exec 1>&2;
set -umeTE; set +f; set -o pipefail;
rm -f ${stderr@Q} &
disown -a
"
}

on_ctrlc() {
    1>&2 echo -e "\x1b[1;38;2;253;67;83m\rAborted with Ctrl-C\x1b[0m"
    exit 130
}
trap on_exit exit
trap on_ctrlc hup
trap on_ctrlc int
trap on_ctrlc bus
trap on_ctrlc segv
trap on_ctrlc sys

declare -a argv=(${@})
declare -i argc=${#argv[@]}
declare code=0

if git_repo_path=$(2>${stderr} git rev-parse --show-toplevel); then
    code=0
else
    code=$?
fi
if [ ${code} -ne 0 ]; then
    1>&2 echo -e "\x1b[0m\x1b[1;48;2;253;67;83m[${script_name} warning]\x1b[7m $(pwd) is not under git version control\x1b[0m"
    exit 1
fi

declare -- target=""
declare -- name=""

declare -a targets=()

declare -- rust_toolchain_path="${git_repo_path}/rust-toolchain.toml"

if [ -f "${rust_toolchain_path}" ] && [ -r "${rust_toolchain_path}" ] && [ -s "${rust_toolchain_path}" ]; then
    targets=($(jaq -r '.toolchain.targets[]' "${rust_toolchain_path}"))
else
    targets=($(rustup target list --installed))
    # targets=($(rustup show | gawk 'BEGIN { PROCINFO["sorted_in"]="@ind_str_asc"; delete TARGETS; can_capture=0; } { if (can_capture) { TARGETS[$NF]=$NF; } else if ($0 ~ /installed\s+targets:/) {can_capture=1;}  } END { for (name in TARGETS) {print(name) } }'))
fi

echo "${targets[*]}" | parallel rustup target add "{}"
echo "${targets[*]}" | parallel cargo zigbuild --target "{}"
