default: (run)

alias r := run 
alias b := build 
alias c := clean

build:
  @echo '[JUST] Web build with `npm`: '
  just lib/ bw
  npm run build

run: build
  @echo '[JUST] Run with `npm`: '
  npm run start

clean:
  @echo '[JUST] Remove `build` directory: '
  rm -r ./build
