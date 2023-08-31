# Org-Mode AST Investigation Tool
This repository contains a slapdash tool to make visualizing the abstract syntax tree of an org-mode document easier. Write your org-mode source into the top text box, and below on the right it will create a clickable tree of the AST. When you click on a node, the contents of that node will be highlighted on the left.

![Screenshot showing the interface to the org-mode abstract syntax tree investigation tool.](readme/screenshot.png?raw=true "Org-mode investigation tool interface")

## Running
Running in docker is the recommended way to run this. It creates a consistent working environment, without impacting (or requiring you to install) emacs, org-mode, or rust.
### Docker
First we need to build the docker container. On the first run, this will pull the emacs and org-mode source code so this build will take a while the first time. After that, subsequent builds should be fast because docker caches the layers.

```bash
# from the root of this repository:
make --directory=docker
```

Next we need to launch the server:
```bash
docker run --init --rm --publish 3000:3000/tcp org-investigation
```

This launches a server listening on port 3000, so pop open your browser to http://127.0.0.1:3000/ to access the web interface.

(alternatively, you can run the `scripts/launch_docker.bash` script which performs these two steps.)
### No docker
You will need a fully functional rust setup with nightly installed (due to the use of exit_status_error). Then from the root of this repo you can launch the server by running:

```bash
cargo run --release
```

It will use your installed version of emacs and org-mode which may differ from what the docker users are using.

This launches a server listening on port 3000, so pop open your browser to http://127.0.0.1:3000/ to access the web interface.
