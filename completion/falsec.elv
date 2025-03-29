
use builtin;
use str;

set edit:completion:arg-completer[falsec] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'falsec'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'falsec'= {
            cand -c 'The path to the configuration file'
            cand --config 'The path to the configuration file'
            cand -d 'Enable debug output'
            cand --debug 'Enable debug output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand run 'Execute a FALSE program'
            cand compile 'Compile a FALSE program'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'falsec;run'= {
            cand --type-safety 'type-safety'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'falsec;compile'= {
            cand --type-safety 'type-safety'
            cand --dump-asm 'The path to the intermediary assembly'
            cand -o 'The path to the compiled FALSE program'
            cand --out 'The path to the compiled FALSE program'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'falsec;help'= {
            cand run 'Execute a FALSE program'
            cand compile 'Compile a FALSE program'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'falsec;help;run'= {
        }
        &'falsec;help;compile'= {
        }
        &'falsec;help;help'= {
        }
    ]
    $completions[$command]
}
