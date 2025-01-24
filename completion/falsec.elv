
use builtin;
use str;

set edit:completion:arg-completer[falsec-cli] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'falsec-cli'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'falsec-cli'= {
            cand -c 'c'
            cand --config 'config'
            cand -d 'd'
            cand --debug 'debug'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand test 'does testing things'
            cand run 'Execute a FALSE program'
            cand compile 'Compile a FALSE program'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'falsec-cli;test'= {
            cand -l 'lists test values'
            cand --list 'lists test values'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'falsec-cli;run'= {
            cand --type-safety 'type-safety'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'falsec-cli;compile'= {
            cand --type-safety 'type-safety'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'falsec-cli;help'= {
            cand test 'does testing things'
            cand run 'Execute a FALSE program'
            cand compile 'Compile a FALSE program'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'falsec-cli;help;test'= {
        }
        &'falsec-cli;help;run'= {
        }
        &'falsec-cli;help;compile'= {
        }
        &'falsec-cli;help;help'= {
        }
    ]
    $completions[$command]
}
