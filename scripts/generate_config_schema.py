#!/usr/bin/env python

if __name__ == '__main__':
    import os
    import subprocess
    from pathlib import Path

    import git

    repo = git.Repo(os.getcwd(), search_parent_directories=True).working_dir
    schema_path = Path(repo) / 'config_schema.json'

    subprocess.run([
        'cargo', 'run', '--release', '--features=schema', '--package', 'falsec-types', '--bin',
        'generate-json-schema', '--',
        '--output', str(schema_path.resolve())
    ], check=True)
