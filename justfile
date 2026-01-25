count:
    tokei . --exclude client/web/drift_worker.js --exclude client/web/drift_worker.min.js

merge_and_push:
    git checkout main
    git merge dev
    git push
    git checkout dev

# Format Rust code
rsfmt:
    @cargo fmt || echo "rust is not installed. Ignored"

# Format protobuf files
buffmt:
    @buf format --write || echo "buf is not installed. Ignored"

# Format Python scripts
pyfmt:
    @ruff format || echo "ruff is not installed. Ignored"

# Format Dart code
dartfmt:
    @cd client && dart format lib/ || echo "dart is not installed. Ignored"

# Format web-panel (TypeScript/JavaScript)
webfmt:
    @cd server/web-panel && pnpm run format || echo "pnpm is not installed. Ignored"

typos:
    @typos|| echo "typos checker is not installed. Ignored"

# Format all code - run as pre-commit
pre-commit: typos rsfmt buffmt pyfmt dartfmt webfmt
