FROM alpine:3.17 AS build
RUN apk add --no-cache build-base musl-dev git autoconf make texinfo gnutls-dev ncurses-dev gawk


FROM build AS build-emacs
ARG EMACS_VERSION=emacs-29.1
RUN git clone --depth 1 --branch $EMACS_VERSION https://git.savannah.gnu.org/git/emacs.git /root/emacs
WORKDIR /root/emacs
RUN mkdir /root/dist
RUN ./autogen.sh
RUN ./configure --prefix /usr --without-x --without-sound
RUN make
RUN make DESTDIR="/root/dist" install


FROM build AS build-org-mode
ARG ORG_VERSION=163bafb43dcc2bc94a2c7ccaa77d3d1dd488f1af
COPY --from=build-emacs /root/dist/ /
RUN mkdir /root/dist
# Savannah does not allow fetching specific revisions, so we're going to have to put unnecessary load on their server by cloning main and then checking out the revision we want.
RUN git clone https://git.savannah.gnu.org/git/emacs/org-mode.git /root/org-mode && git -C /root/org-mode checkout $ORG_VERSION
# RUN mkdir /root/org-mode && git -C /root/org-mode init --initial-branch=main && git -C /root/org-mode remote add origin https://git.savannah.gnu.org/git/emacs/org-mode.git && git -C /root/org-mode fetch origin $ORG_VERSION && git -C /root/org-mode checkout FETCH_HEAD
WORKDIR /root/org-mode
RUN make compile
RUN make DESTDIR="/root/dist" install


FROM rustlang/rust:nightly-alpine3.17 AS build-org-investigation
RUN apk add --no-cache musl-dev
RUN mkdir /root/org-investigation
WORKDIR /root/org-investigation
COPY . .
RUN CARGO_TARGET_DIR=/target cargo build --profile release-lto


FROM alpine:3.17 AS run
ENV LANG=en_US.UTF-8
RUN apk add --no-cache ncurses gnutls
COPY --from=build-emacs /root/dist/ /
COPY --from=build-org-mode /root/dist/ /
COPY --from=build-org-investigation /target/release-lto/org_ownership_investigation /usr/bin/
COPY static /opt/org-investigation/static
WORKDIR /opt/org-investigation
CMD ["/usr/bin/org_ownership_investigation"]
