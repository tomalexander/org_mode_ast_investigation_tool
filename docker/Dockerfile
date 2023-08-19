FROM alpine:3.17 AS build
RUN apk add --no-cache build-base musl-dev git autoconf make texinfo gnutls-dev ncurses-dev gawk


FROM build AS build-emacs
RUN git clone --depth 1 --branch emacs-29.1 https://git.savannah.gnu.org/git/emacs.git /root/emacs
WORKDIR /root/emacs
RUN mkdir /root/dist
RUN ./autogen.sh
RUN ./configure --prefix /usr --without-x --without-sound
RUN make
RUN make DESTDIR="/root/dist" install


FROM build AS build-org-mode
COPY --from=build-emacs /root/dist/ /
RUN mkdir /root/dist
RUN mkdir /root/org-mode && git -C /root/org-mode init --initial-branch=main && git -C /root/org-mode remote add origin https://git.savannah.gnu.org/git/emacs/org-mode.git && git -C /root/org-mode fetch origin 3cbd9f423385bf725dc964a5cff573bba17db3ff && git -C /root/org-mode checkout FETCH_HEAD
WORKDIR /root/org-mode
RUN make compile
RUN make DESTDIR="/root/dist" install


FROM rustlang/rust:nightly-alpine3.17 AS build-org-investigation
RUN apk add --no-cache musl-dev
RUN mkdir /root/org-investigation
WORKDIR /root/org-investigation
COPY . .
RUN CARGO_TARGET_DIR=/target cargo build --release


FROM alpine:3.17 AS run
RUN apk add --no-cache ncurses gnutls
COPY --from=build-emacs /root/dist/ /
COPY --from=build-org-mode /root/dist/ /
COPY --from=build-org-investigation /target/release/org_ownership_investigation /usr/bin/
COPY static /opt/org-investigation/static
WORKDIR /opt/org-investigation
CMD ["/usr/bin/org_ownership_investigation"]
