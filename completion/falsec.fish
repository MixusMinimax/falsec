# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_falsec_global_optspecs
	string join \n c/config= d/debug h/help V/version
end

function __fish_falsec_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_falsec_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_falsec_using_subcommand
	set -l cmd (__fish_falsec_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c falsec -n "__fish_falsec_needs_command" -s c -l config -d 'The path to the configuration file' -r -F
complete -c falsec -n "__fish_falsec_needs_command" -s d -l debug -d 'Enable debug output'
complete -c falsec -n "__fish_falsec_needs_command" -s h -l help -d 'Print help'
complete -c falsec -n "__fish_falsec_needs_command" -s V -l version -d 'Print version'
complete -c falsec -n "__fish_falsec_needs_command" -a "run" -d 'Execute a FALSE program'
complete -c falsec -n "__fish_falsec_needs_command" -a "compile" -d 'Compile a FALSE program'
complete -c falsec -n "__fish_falsec_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c falsec -n "__fish_falsec_using_subcommand run" -l type-safety -r -f -a "none\t'No type safety checks are performed'
lambda\t'When trying to execute a lambda, make sure that the popped value is a lambda'
lambda-and-var\t'Include all checks from [TypeSafety::Lambda], and make sure that when storing or loading a variable, the popped value is a variable name'
full\t'Include all checks from [TypeSafety::LambdaAndVar], and ensure that only integers can be used for arithmetic operations'"
complete -c falsec -n "__fish_falsec_using_subcommand run" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c falsec -n "__fish_falsec_using_subcommand run" -s V -l version -d 'Print version'
complete -c falsec -n "__fish_falsec_using_subcommand compile" -l type-safety -r -f -a "none\t'No type safety checks are performed'
lambda\t'When trying to execute a lambda, make sure that the popped value is a lambda'
lambda-and-var\t'Include all checks from [TypeSafety::Lambda], and make sure that when storing or loading a variable, the popped value is a variable name'
full\t'Include all checks from [TypeSafety::LambdaAndVar], and ensure that only integers can be used for arithmetic operations'"
complete -c falsec -n "__fish_falsec_using_subcommand compile" -l dump-asm -d 'The path to the intermediary assembly' -r -F
complete -c falsec -n "__fish_falsec_using_subcommand compile" -s o -l out -d 'The path to the compiled FALSE program' -r -F
complete -c falsec -n "__fish_falsec_using_subcommand compile" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c falsec -n "__fish_falsec_using_subcommand compile" -s V -l version -d 'Print version'
complete -c falsec -n "__fish_falsec_using_subcommand help; and not __fish_seen_subcommand_from run compile help" -f -a "run" -d 'Execute a FALSE program'
complete -c falsec -n "__fish_falsec_using_subcommand help; and not __fish_seen_subcommand_from run compile help" -f -a "compile" -d 'Compile a FALSE program'
complete -c falsec -n "__fish_falsec_using_subcommand help; and not __fish_seen_subcommand_from run compile help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
