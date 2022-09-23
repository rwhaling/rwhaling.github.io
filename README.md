# rwhaling.github.io

## setup
- install Zola via homebrew: `brew install zola`

## dev cycle
- `zola serve` and navigate to http://127.0.0.1:1111

## publishing
no automatic rendering yet
- publishes from `docs/` dir in `main`
- make sure you are under `./whaling.dev`
- `zola build`
- `rm -rf ../docs/*` 
- `cp -r public/* ../docs/`
- `git commit`
- `git push`
