notes.md contains miscelaneous notes, instructions, and links that I found useful while 

# Project setup
Tauri is under active development, and I started using it a couple days after the beta release.  I mainly followed the [Tauri Setup](https://tauri.studio/en/docs/getting-started/setup-linux) guide, but ran into a couple issues with the project structure and yarn 2.

Install the system dependencies, rust, and node.js as described here: https://tauri.studio/en/docs/getting-started/setup-linux

## Windows Subsystem for Linux
Running tauri under WSL had additional requirements, including updating WSL to 2, installing vcXsrv, allowing vcXsrv to accept connections on local / public networks, and exporting DISPLAY as described in the tauri docs.

Hot reloading works if the project is checked out under the linux filesystem.  The linux filesystem is accessible in windows under `\\wsl$\Ubuntu-20.04`.

## Install NVM
nvm install node --latest-npm
nvm use node

## Install yarn
https://yarnpkg.com/getting-started/install
`npm install -g yarn` to install yarn globally (yarn 1, probably)
`yarn --version` to check global yarn version (OK if it's something like 1.22.0)
`yarn set version berry` to switch project's yarn to 2

Yarn commands: https://yarnpkg.com/getting-started/usage
Not using zero installs, so cloners need to run `yarn install` to pull dependencies.

## Create a project
`yarn init` to start a new yarn project

Yarn2 compatibility with vue-cli is still an open issue - https://github.com/vuejs/vue-cli/issues/5135, but I want to use yarn 2 to have the option to split the application up into workspaces.

Manually specifying dependencies in `.yarnrc.yml worked - compiled from several suggestions in vue-cli#5135, plus errors that `yarn serve` reported.  Iterated by editing `.yarnrc.yaml`, running `yarn install`, then `yarn serve` in `/packages/frontend`.

`.yarnrc.yml`:
```yml
yarnPath: ".yarn/releases/yarn-berry.cjs"

packageExtensions:
  "@vue/babel-preset-app@*":
    peerDependencies:
      vue: ^2.6.10
  "@vue/cli-plugin-typescript@*":
    dependencies:
      babel-loader: "*"
  "@vue/cli-service@*":
    peerDependencies:
      "@vue/babel-preset-app": "*"
      "@vue/cli-plugin-babel": "*"
      "@vue/cli-plugin-e2e-nightwatch": "*"
      "@vue/cli-plugin-eslint": "*"
      "@vue/cli-plugin-pwa": "*"
      "@vue/cli-plugin-typescript": "*"
      "@vue/cli-plugin-unit-jest": "*"
      vue-cli-plugin-tauri: "*"
  "@vue/babel-plugin-jsx@*":
    peerDependencies:
      "@babel/core": "*"
  babel-loader@*:
    dependencies:
      "@babel/core": "*"
  fork-ts-checker-webpack-plugin@*:
    dependencies:
      vue-template-compiler: "*"
    peerDependencies:
      typescript: "*"
  vue-eslint-parser@*:
    dependencies:
      babel-eslint: "*"
  "@yzfe/vue-svgicon@*":
    peerDependencies:
      "@yzfe/svgicon-loader": "*"
  vue-loader@*:
    peerDependencies:
      "@vue/compiler-sfc": "*"
      webpack: "*"
```

Create vue frontend using [vue-cli-plugin-tauri](https://github.com/tauri-apps/vue-cli-plugin-tauri):
`yarn add @vue/cli` to install the vue cli
`cargo install tauri-bundler`
`yarn vue create .` - select vue3, yarn
`yarn serve` to test that it's working - vue page should be available on http://localhost:8080

Install the Vue CLI plugin tauri
`vue add tauri`
Adjust package versions in src-tauri/Cargo.toml
`yarn tauri:serve` to check that it's working

Add tauri cli: https://tauri.studio/en/docs/usage/development/integration
`yarn add @tauri-apps/cli`
`yarn tauri info` to check that the tauri cli is working

## Updates
`yarn add @tauri-apps/cli` adds the tauri command.  RC builds are published frequently, so `yarn upgrade @tauri-apps/cli @tauri-apps/api --latest` updates tauri if `yarn tauri ...` dies after printing `Dowloading Rust CLI`.  `yarn upgrade vue-cli-plugin-tauri --latest` upgrades the vue-cli-plugin-tauri.  Need to update the tauri version in Cargo.toml too.  `cargo outdated -r tauri` lists info about the latest cargo package, and `yarn outdated @tauri-apps/cli` prints info about the yarn packages.

## Sample Project Layouts
Layout:
src-tauri/ - tauri application
src-tauri/src/ - rust sources

Couple different structures - yarn has the concept of 'workspaces' (found under /packages/...), which allow an application to be structured into dependent packages.

https://github.com/Akryum/guijs places the tauri application under /packages/@guijs/tauri-app/src-tauri, the vue front-end code under /packages/@guijs/frontend-core, a node server for the API under /packages/@guijs/server-core (Apollo for a GraphQL API)

Try the [Bridge](https://tauri.studio/en/docs/usage/patterns/bridge) pattern?  Tauri defines `commands` that can send/receive data.

