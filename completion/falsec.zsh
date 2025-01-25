#compdef falsec

autoload -U is-at-least

_falsec() {
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
":: :_falsec_commands" \
"*::: :->falsec" \
&& ret=0
    case $state in
    (falsec)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:falsec-command-$line[2]:"
        case $line[2] in
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
'--dump-asm=[The path to the intermediary assembly]:FILE:_default' \
'-o+[The path to the compiled FALSE program]:FILE:_default' \
'--out=[The path to the compiled FALSE program]:FILE:_default' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
':program -- The path to the FALSE program to execute:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_falsec__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:falsec-help-command-$line[1]:"
        case $line[1] in
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

(( $+functions[_falsec_commands] )) ||
_falsec_commands() {
    local commands; commands=(
'run:Execute a FALSE program' \
'compile:Compile a FALSE program' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'falsec commands' commands "$@"
}
(( $+functions[_falsec__compile_commands] )) ||
_falsec__compile_commands() {
    local commands; commands=()
    _describe -t commands 'falsec compile commands' commands "$@"
}
(( $+functions[_falsec__help_commands] )) ||
_falsec__help_commands() {
    local commands; commands=(
'run:Execute a FALSE program' \
'compile:Compile a FALSE program' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'falsec help commands' commands "$@"
}
(( $+functions[_falsec__help__compile_commands] )) ||
_falsec__help__compile_commands() {
    local commands; commands=()
    _describe -t commands 'falsec help compile commands' commands "$@"
}
(( $+functions[_falsec__help__help_commands] )) ||
_falsec__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'falsec help help commands' commands "$@"
}
(( $+functions[_falsec__help__run_commands] )) ||
_falsec__help__run_commands() {
    local commands; commands=()
    _describe -t commands 'falsec help run commands' commands "$@"
}
(( $+functions[_falsec__run_commands] )) ||
_falsec__run_commands() {
    local commands; commands=()
    _describe -t commands 'falsec run commands' commands "$@"
}

if [ "$funcstack[1]" = "_falsec" ]; then
    _falsec "$@"
else
    compdef _falsec falsec
fi
