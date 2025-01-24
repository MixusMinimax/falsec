
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'falsec-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'falsec-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'falsec-cli' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'config')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'd')
            [CompletionResult]::new('--debug', '--debug', [CompletionResultType]::ParameterName, 'debug')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'does testing things')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute a FALSE program')
            [CompletionResult]::new('compile', 'compile', [CompletionResultType]::ParameterValue, 'Compile a FALSE program')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'falsec-cli;test' {
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'lists test values')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'lists test values')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'falsec-cli;run' {
            [CompletionResult]::new('--type-safety', '--type-safety', [CompletionResultType]::ParameterName, 'type-safety')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'falsec-cli;compile' {
            [CompletionResult]::new('--type-safety', '--type-safety', [CompletionResultType]::ParameterName, 'type-safety')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'falsec-cli;help' {
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'does testing things')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute a FALSE program')
            [CompletionResult]::new('compile', 'compile', [CompletionResultType]::ParameterValue, 'Compile a FALSE program')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'falsec-cli;help;test' {
            break
        }
        'falsec-cli;help;run' {
            break
        }
        'falsec-cli;help;compile' {
            break
        }
        'falsec-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
