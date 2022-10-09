# rwhaling.github.io

## setup
- install Zola 0.16.1 via homebrew: `brew install zola`

## dev cycle
- `zola serve` and navigate to http://127.0.0.1:1111

## publishing
no automatic rendering yet
- publishes from `docs/` dir in `main`
- make sure you are in the root of the repo
- `zola build --output-dir docs` (will prompt you to delete/overwrite)
- `echo "whaling.dev" > docs/CNAME` (because the overwrite kills the CNAME file, sigh)
- `git commit`
- `git push`

(or just run `publish.sh` instead of the first two steps)