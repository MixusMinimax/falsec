#!/usr/bin/env python

if __name__ == '__main__':
    import os
    import subprocess
    from pathlib import Path

    import git

    shells = [
        ('zsh', 'falsec.zsh'),
        ('bash', 'falsec.bash'),
        ('fish', 'falsec.fish'),
        ('powershell', 'falsec.ps1'),
        ('elvish', 'falsec.elv'),
    ]

    repo = git.Repo(os.getcwd(), search_parent_directories=True).working_dir
    completion_folder = Path(repo) / 'completion'

    for (shell, p) in shells:
        subprocess.run([
            'cargo', 'run', '-q', '--release', '--no-default-features', '--features=completions', '--package',
            'falsec-cli', '--bin', 'print-completions', '--',
            f'--generator={shell}', str(completion_folder / p)
        ], check=True)
