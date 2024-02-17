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
    __nk_tmp_output=$(mktemp) || {
        declare __nk_return_value="$?"
        echo 'nk::run_for_output failed to create temporary file'
        return "$__nk_return_value"
    }

    # run
    declare __nk_return_value='0'
    "${__nk_command[@]}" >"$__nk_tmp_output" 2>&1 || __nk_return_value="$?"

    # read output into var
    declare __nk_output
    IFS= read -r -d '' '__nk_output' < "$__nk_tmp_output"
    rm "$__nk_tmp_output"

    # export with given name
    # TODO: would prefer a global instead of an env var, but the defult verion of bash on macos doesn't support this yet: declare -g
    export "$__nk_output_var"="$__nk_output"

    # preserve ret code
    return "$__nk_return_value"
}

nk::run_for_separated_output() {
    if [[ "$#" -lt 3 ]]; then
        echo 'usage: nk::run_for_separated_output <stdout_var> <stderr_var> <command> [<args>...]'
        return 1
    fi
    declare __nk_stdout_var="$1"
    declare __nk_stderr_var="$2"
    declare -a __nk_command=("${@:3}")

    # create temp stdout file
    declare __nk_tmp_stdout
    __nk_tmp_stdout=$(mktemp) || {
        declare __nk_return_value="$?"
        echo 'nk::run_for_separated_output failed to create temporary stdout file'
        return "$__nk_return_value"
    }
    # create temp stderr file
    declare __nk_tmp_stderr
    __nk_tmp_stderr=$(mktemp) || {
        declare __nk_return_value="$?"
        echo 'nk::run_for_separated_output failed to create temporary stderr file'
        return "$__nk_return_value"
    }

    # run
    declare __nk_return_value='0'
    "${__nk_command[@]}" > "$__nk_tmp_stdout" 2> "$__nk_tmp_stderr" || __nk_return_value="$?"

    # read stdout into var
    declare __nk_stdout
    IFS= read -r -d '' '__nk_stdout' < "$__nk_tmp_stdout"
    rm "$__nk_tmp_stdout"

    # read stderr into var
    declare __nk_stderr
    IFS= read -r -d '' '__nk_stderr' < "$__nk_tmp_stderr"
    rm "$__nk_tmp_stderr"

    # export with given name
    # TODO: would prefer a global instead of an env var, but the defult verion of bash on macos doesn't support this yet: declare -g
    export "$__nk_stdout_var"="$__nk_stdout"
    export "$__nk_stderr_var"="$__nk_stderr"

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
    # TODO: should probably keep manual summaries (ie. at least for one off error checks like a package not existing)

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

nk::array::contains() {
    if [[ "$#" == '0' ]]; then
        echo 'usage: nk::array::contains <element> <array>...' >&2
        echo "   ie. nk::array::contains \"\$program\" \"\${programs[@]}\"" >&2
        return 1
    fi

    declare value="$1"
    for element in "${@:2}"; do
        if [[ "$element" == "$value" ]]; then
            return 0
        fi
    done

    return 1
}

nk::error() {
    echo "$1"
    echo "${@:2}" >&2
}
