count:
    tokei . --exclude client/web/drift_worker.js --exclude client/web/drift_worker.min.js

merge_and_push:
    git checkout main
    git merge dev
    git push
    git checkout dev
