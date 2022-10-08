#!/usr/bin/env bash

nk::run_for_output() {
    if [[ "$#" -lt 2 ]]; then
        echo 'usage: nk::run_for_output <output_var> <command> [<args>...]'
        return 1
    fi
    declare __nk_output_var="$1"
    declare -a __nk_command=("${@:2}")

    # create temp output file
    declare __nk_tmp_output
    __nk_tmp_output=$(mktemp)

    # run
    declare __nk_return_value='0'
    "${__nk_command[@]}" >"$__nk_tmp_output" 2>&1 || __nk_return_value="$?"

    # read output into var
    declare __nk_output
    __nk_output="$(cat "$__nk_tmp_output")"
    rm "$__nk_tmp_output"

    # export with given name
    # TODO: would prefer a global instead of an env var, but the defult verion of bash on macos doesn't support this yet: declare -g
    export "$__nk_output_var"="$__nk_output"

    # preserve ret code
    return "$__nk_return_value"
}

nk::log_result() {
    # TODO: have proper named args
    # TODO: validate status/changed
    declare status="$1"
    declare changed="$2"
    declare description="$3"
    declare output="$4"

    jq \
        --null-input \
        --compact-output \
        --arg 'status' "$status" \
        --argjson 'changed' "$changed" \
        --arg 'description' "$description" \
        --arg 'output' "$output" \
        '{
            "status": $status,
            "changed": $changed,
            "description": $description,
            "output": $output
        }'
}
