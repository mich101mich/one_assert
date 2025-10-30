#!/bin/bash -e

base_dir="$(realpath "$(dirname "$0")")"
sub_directories=""
is_proc_macro=1
msrv_overrides="trybuild@1.0.109 toml@0.9.6 toml_datetime@0.7.1 toml_parser@1.0.2 toml_writer@1.0.2 serde_spanned@1.0.1"

export base_dir sub_directories is_proc_macro msrv_overrides

"${base_dir}/submodules/test_script/test.sh" "$@"
