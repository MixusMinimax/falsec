
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'falsec' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'falsec'
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
        'falsec' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'The path to the configuration file')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'The path to the configuration file')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Enable debug output')
            [CompletionResult]::new('--debug', '--debug', [CompletionResultType]::ParameterName, 'Enable debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute a FALSE program')
            [CompletionResult]::new('compile', 'compile', [CompletionResultType]::ParameterValue, 'Compile a FALSE program')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'falsec;run' {
            [CompletionResult]::new('--type-safety', '--type-safety', [CompletionResultType]::ParameterName, 'type-safety')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'falsec;compile' {
            [CompletionResult]::new('--type-safety', '--type-safety', [CompletionResultType]::ParameterName, 'type-safety')
            [CompletionResult]::new('--dump-asm', '--dump-asm', [CompletionResultType]::ParameterName, 'The path to the intermediary assembly')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'The path to the compiled FALSE program')
            [CompletionResult]::new('--out', '--out', [CompletionResultType]::ParameterName, 'The path to the compiled FALSE program')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'falsec;help' {
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute a FALSE program')
            [CompletionResult]::new('compile', 'compile', [CompletionResultType]::ParameterValue, 'Compile a FALSE program')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'falsec;help;run' {
            break
        }
        'falsec;help;compile' {
            break
        }
        'falsec;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
