#compdef falsec-cli

autoload -U is-at-least

_falsec-cli() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-c+[]:FILE:_files' \
'--config=[]:FILE:_files' \
'*-d[]' \
'*--debug[]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
'::name:_default' \
":: :_falsec-cli_commands" \
"*::: :->falsec-cli" \
&& ret=0
    case $state in
    (falsec-cli)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:falsec-cli-command-$line[2]:"
        case $line[2] in
            (test)
_arguments "${_arguments_options[@]}" : \
'-l[lists test values]' \
'--list[lists test values]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(run)
_arguments "${_arguments_options[@]}" : \
'--type-safety=[]:TYPE:((none\:"No type safety checks are performed"
lambda\:"When trying to execute a lambda, make sure that the popped value is a lambda"
lambda-and-var\:"Include all checks from \[TypeSafety\:\:Lambda\], and make sure that when storing or loading a variable, the popped value is a variable name"
full\:"Include all checks from \[TypeSafety\:\:LambdaAndVar\], and ensure that only integers can be used for arithmetic operations"))' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':program -- The path to the FALSE program to execute:_default' \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
'--type-safety=[]:TYPE:((none\:"No type safety checks are performed"
lambda\:"When trying to execute a lambda, make sure that the popped value is a lambda"
lambda-and-var\:"Include all checks from \[TypeSafety\:\:Lambda\], and make sure that when storing or loading a variable, the popped value is a variable name"
full\:"Include all checks from \[TypeSafety\:\:LambdaAndVar\], and ensure that only integers can be used for arithmetic operations"))' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':program -- The path to the FALSE program to execute:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_falsec-cli__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:falsec-cli-help-command-$line[1]:"
        case $line[1] in
            (test)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_falsec-cli_commands] )) ||
_falsec-cli_commands() {
    local commands; commands=(
'test:does testing things' \
'run:Execute a FALSE program' \
'compile:Compile a FALSE program' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'falsec-cli commands' commands "$@"
}
(( $+functions[_falsec-cli__compile_commands] )) ||
_falsec-cli__compile_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli compile commands' commands "$@"
}
(( $+functions[_falsec-cli__help_commands] )) ||
_falsec-cli__help_commands() {
    local commands; commands=(
'test:does testing things' \
'run:Execute a FALSE program' \
'compile:Compile a FALSE program' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'falsec-cli help commands' commands "$@"
}
(( $+functions[_falsec-cli__help__compile_commands] )) ||
_falsec-cli__help__compile_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli help compile commands' commands "$@"
}
(( $+functions[_falsec-cli__help__help_commands] )) ||
_falsec-cli__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli help help commands' commands "$@"
}
(( $+functions[_falsec-cli__help__run_commands] )) ||
_falsec-cli__help__run_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli help run commands' commands "$@"
}
(( $+functions[_falsec-cli__help__test_commands] )) ||
_falsec-cli__help__test_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli help test commands' commands "$@"
}
(( $+functions[_falsec-cli__run_commands] )) ||
_falsec-cli__run_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli run commands' commands "$@"
}
(( $+functions[_falsec-cli__test_commands] )) ||
_falsec-cli__test_commands() {
    local commands; commands=()
    _describe -t commands 'falsec-cli test commands' commands "$@"
}

if [ "$funcstack[1]" = "_falsec-cli" ]; then
    _falsec-cli "$@"
else
    compdef _falsec-cli falsec-cli
fi
